//! A complete SBML extension, written the way a package outside this crate is written.
//!
//! This example is the reference for the `extension` module of `biodivine-lib-sbml`: it declares an
//! SBML package in a few dozen lines and then uses every part of the extension surface. The package
//! ("myext") adds two things to the core object model, the way a real package does:
//!
//!  * **an attribute of the package's own namespace on a core element** - `myext:strictness` on
//!    `<model>`, a number between 0 and 1;
//!  * **an element type of its own namespace** - `<myext:tuningParameter>`, collected in
//!    `<myext:listOfTuningParameters>`, each carrying an `id` and a `value`.
//!
//! On top of that it specifies **two rules** and takes part in the validation of a document: one
//! rule its own specification defines (`myext-10101`, an error: every tuning parameter must set its
//! `value`) and one library diagnostic that no specification owns (`myext-empty-list`, an
//! informational note: a `listOfTuningParameters` that is present but empty).
//!
//! The example then walks through the whole life cycle of a package-aware document:
//!
//!  1. a document that does not declare the package is **read** and the cast to the package's view
//!     is refused with [`ExtensionError::NotDeclared`](biodivine_lib_sbml::ExtensionError);
//!  2. the **editing-side cast** declares the package on `<sbml>` (namespace + `required`) and
//!     produces the view;
//!  3. the package's **attribute and element are written and read back** - including the element
//!     type, built from its declared identity;
//!  4. the document is **validated with the package's rules injected**: the good document is silent,
//!     and a deliberately broken one reports `myext-10101` and the empty-list note;
//!  5. the document is written to a file and read back, and the package data survives the **round
//!     trip**.
//!
//! Run it with `cargo run --example extension-example`. `tests/extension-example.rs` includes this
//! file and runs the same code path, so the example cannot rot, and the Biodivine book includes it
//! verbatim.
//!
//! Nothing here is a helper of the crate: the wrappers come from the `XmlWrapper` derive, the
//! accessors from the public XML layer, the cast from `ExtensionCast`, the rules from `RuleSet` and
//! the injection from `ExtensionValidation`.

use std::error::Error;

use biodivine_lib_sbml::extension::{ExtensionCast, ExtensionOf, ExtensionValidation};
use biodivine_lib_sbml::xml::{
    OptionalSbmlChild, OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty,
    XmlChildDefault, XmlElement, XmlList, XmlWrapper,
};
use biodivine_lib_sbml::{
    CancellationCheck, RuleDescriptor, RuleKind, RuleSet, Sbml, SbmlIssue, SbmlIssueSeverity,
    SbmlResult, URL_SBML_CORE,
};
use sbml_macros::XmlWrapper;

/// The namespace of the package: the conventional prefix this crate writes its names with, and the
/// URL that identifies the namespace.
///
/// The URL is written in the shape a package of the SBML standard uses,
/// `.../level3/version2/<package>/version1`. That is not cosmetic: an element or an attribute in a
/// namespace that is neither a namespace of the standard nor a foreign one is reported by rule
/// 10102, so a package that adds content to an SBML document has to declare a namespace of this
/// shape.
pub const MY_EXTENSION: (&str, &str) = (
    "myext",
    "http://www.sbml.org/sbml/level3/version2/myext/version1",
);

// ---------------------------------------------------------------------------------------------
// The elements the package adds to the object model.
// ---------------------------------------------------------------------------------------------

/// The package's `tuningParameter` element.
///
/// A wrapper of a package is declared exactly like a wrapper of the core specification:
/// `#[derive(XmlWrapper)]` with an `#[sbml_object(..)]` attribute naming the element and the
/// namespace. The derive writes the identity (the element name, the namespace, and the `xsi:type` of
/// a discriminated type) and the `new_empty(document)` constructor, so the element a value is built
/// from and the element the type recognizes are the same by construction.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "tuningParameter", namespace = MY_EXTENSION)]
pub struct TuningParameter(XmlElement);

impl TuningParameter {
    /// The `id` attribute of the element, in the namespace of the package.
    ///
    /// The four arguments of [`OptionalSbmlProperty::new`] say where the attribute lives (the
    /// namespace of the package) and in which namespace the element it is read from lives (the
    /// package too, for a package element), which is what tells the accessor that the attribute may
    /// be written with or without its prefix.
    pub fn id(&self) -> OptionalSbmlProperty<'_, String> {
        OptionalSbmlProperty::new(self.xml_element(), "id", MY_EXTENSION, MY_EXTENSION)
    }

    /// The `value` attribute of the element, in the namespace of the package.
    pub fn value(&self) -> OptionalSbmlProperty<'_, f64> {
        OptionalSbmlProperty::new(self.xml_element(), "value", MY_EXTENSION, MY_EXTENSION)
    }
}

// ---------------------------------------------------------------------------------------------
// The view the package adds to a core object.
// ---------------------------------------------------------------------------------------------

/// The package's view of a [`Model`](biodivine_lib_sbml::core::Model).
///
/// A view is a thin wrapper: it holds the core object (a cheap handle to a document element) and
/// reaches the package data through the accessors of the XML layer. `ExtensionOf` names the package
/// and converts in both directions, and the cast of `ExtensionCast` is what a caller uses to get a
/// view *checked*: a view cannot be read out of a document that does not declare the package.
#[derive(Debug)]
pub struct MyExtensionModel {
    model: biodivine_lib_sbml::core::Model,
}

impl ExtensionOf<biodivine_lib_sbml::core::Model> for MyExtensionModel {
    const PACKAGE: (&'static str, &'static str) = MY_EXTENSION;

    fn from_object(model: biodivine_lib_sbml::core::Model) -> Self {
        MyExtensionModel { model }
    }

    fn object(&self) -> &biodivine_lib_sbml::core::Model {
        &self.model
    }
}

impl MyExtensionModel {
    /// The `myext:strictness` attribute the package adds to `<model>`.
    ///
    /// The helper of [`ExtensionOf`] fills the package namespace in, so the accessor is one line.
    pub fn strictness(&self) -> OptionalSbmlProperty<'_, f64> {
        self.property("strictness")
    }

    /// The package's `listOfTuningParameters` child of `<model>`.
    pub fn tuning_parameters(&self) -> OptionalSbmlChild<'_, XmlList<TuningParameter>> {
        self.child("listOfTuningParameters")
    }
}

// ---------------------------------------------------------------------------------------------
// The rules of the package.
// ---------------------------------------------------------------------------------------------

/// The rules of the package, and the checks that report them.
///
/// The two rules of this package are one of each kind, and the kind is a property of the *rule*
/// ([`RuleKind`]): `myext-10101` is a rule the package's own specification defines (so the report of
/// a violated document belongs in the comparison against the SBML test suite), while
/// `myext-empty-list` is a diagnostic this package raises on its own initiative, which no
/// specification requires and which must therefore never make a valid document "fail".
pub struct MyExtensionRules {
    rules: RuleSet,
}

impl Default for MyExtensionRules {
    fn default() -> Self {
        MyExtensionRules {
            rules: RuleSet::new(vec![
                RuleDescriptor::new(
                    "myext-10101",
                    RuleKind::SpecRule,
                    SbmlIssueSeverity::Error,
                    "Every `<myext:tuningParameter>` element must set a `value` attribute.",
                ),
                RuleDescriptor::new(
                    "myext-empty-list",
                    RuleKind::LibraryLint,
                    SbmlIssueSeverity::Info,
                    "The package's list of tuning parameters is present but empty.",
                ),
            ]),
        }
    }
}

impl ExtensionValidation for MyExtensionRules {
    fn rules(&self) -> &RuleSet {
        &self.rules
    }

    fn validate(
        &self,
        document: &Sbml,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        // The check accounts for its own steps, so a request to stop reaches the package's walk too.
        check.tick()?;

        let Some(model) = document.model().get() else {
            return Ok(());
        };
        // Whether the package is relevant is the package's own question: this one checks the
        // documents that declare it, and says nothing about the others.
        let Ok(view) = model.cast::<MyExtensionModel>() else {
            return Ok(());
        };
        let Some(parameters) = view.tuning_parameters().get() else {
            return Ok(());
        };

        if parameters.is_empty() {
            issues.push(SbmlIssue::new_in(
                self.rules(),
                "myext-empty-list",
                &model,
                "The document declares `<myext:listOfTuningParameters>` and leaves it empty.",
            ));
        }
        for parameter in parameters.as_vec() {
            check.tick()?;
            if parameter.value().get().is_none() {
                issues.push(SbmlIssue::new_in(
                    self.rules(),
                    "myext-10101",
                    &parameter,
                    "A `<myext:tuningParameter>` element must set its `value` attribute.",
                ));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// The life cycle of a package-aware document.
// ---------------------------------------------------------------------------------------------

/// The document the example starts from: a plain Level 3 Version 2 core document with a model that
/// the package can add its data to.
fn core_document() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="{URL_SBML_CORE}" level="3" version="2">
  <model>
    <listOfCompartments>
      <compartment id="cell" size="1" constant="true"/>
    </listOfCompartments>
  </model>
</sbml>
"#
    )
}

/// A document that declares the package and breaks its rule: the second tuning parameter has no
/// `value`.
fn broken_document() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="{URL_SBML_CORE}" xmlns:myext="{}" myext:required="false" level="3" version="2">
  <model>
    <myext:listOfTuningParameters>
      <myext:tuningParameter myext:id="tp1" myext:value="1.5"/>
      <myext:tuningParameter myext:id="tp2"/>
    </myext:listOfTuningParameters>
  </model>
</sbml>
"#,
        MY_EXTENSION.1
    )
}

/// A document that declares the package and satisfies its specification rule, but carries an empty
/// list of tuning parameters - the diagnostic the package raises on its own initiative.
fn empty_list_document() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="{URL_SBML_CORE}" xmlns:myext="{}" myext:required="false" level="3" version="2">
  <model>
    <myext:listOfTuningParameters/>
  </model>
</sbml>
"#,
        MY_EXTENSION.1
    )
}

/// Runs the whole example and returns the error of whichever step failed.
pub fn run() -> Result<(), Box<dyn Error>> {
    // 1. A document that does not declare the package. The read-side cast refuses to hand out a view
    //    of a package the document does not say it uses.
    let document = Sbml::read_str(&core_document())?;
    let model = document
        .model()
        .get()
        .expect("The example document contains a model.");
    let error = model
        .cast::<MyExtensionModel>()
        .expect_err("The document declares no package.");
    assert!(error.is_not_declared());
    assert_eq!(error.package(), MY_EXTENSION.1);

    // 2. The editing-side cast declares the package on the `<sbml>` element (the namespace and its
    //    `required` attribute) and then produces the view.
    let view = model.cast_or_activate::<MyExtensionModel>()?;
    assert_eq!(
        document.declared_package_prefix(MY_EXTENSION),
        Ok("myext".to_string())
    );

    // 3. The package's own data is written and read back: an attribute of a core element, and an
    //    element of the package's namespace that is built from its declared identity.
    view.strictness().set_some(&0.75);
    assert_eq!(view.strictness().get(), Some(0.75));

    let parameter = TuningParameter::new_empty(document.xml_element().document());
    parameter.id().set_some(&"tp1".to_string());
    parameter.value().set_some(&1.5);
    let parameters = view.tuning_parameters().get_or_create();
    parameters.push(parameter);
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters.get(0).value().get(), Some(1.5));

    // 4. The package's rules take part in the validation of the document. The document above is
    //    good, so the package reports nothing ...
    let rules = MyExtensionRules::default();
    let issues = document.validate_with(&[&rules])?;
    assert!(
        issues.iter().all(|it| !it.rule().starts_with("myext-")),
        "the good document violates no rule of the package: {issues:#?}"
    );

    // ... while a document that breaks a rule of the package is reported under the package's own id,
    // with the kind and the severity its descriptor gives it.
    let broken = Sbml::read_str(&broken_document())?;
    let issues = broken.validate_with(&[&rules])?;
    let violation = issues
        .iter()
        .find(|it| it.rule() == "myext-10101")
        .expect("The package's rule reports the missing `value`.");
    assert_eq!(violation.kind(), RuleKind::SpecRule);
    assert_eq!(violation.severity(), SbmlIssueSeverity::Error);

    // The package's second rule is the other kind: a document that is *valid* still gets the
    // note, and the note is an informational diagnostic of the package rather than a rule of any
    // specification.
    let empty = Sbml::read_str(&empty_list_document())?;
    let issues = empty.validate_with(&[&rules])?;
    let note = issues
        .iter()
        .find(|it| it.rule() == "myext-empty-list")
        .expect("The package's diagnostic reports the empty list.");
    assert_eq!(note.kind(), RuleKind::LibraryLint);
    assert_eq!(note.severity(), SbmlIssueSeverity::Info);

    // A caller that compares against the SBML test suite classifies the reported ids with the rules
    // of *the run*: this crate's table plus the package's. Its rule is then a specification rule,
    // and its diagnostic is not - which is what keeps a package's opinion out of a comparison
    // against expectation files that list specification rule ids.
    let rules_of_the_run = RuleSet::CORE.with(rules.rules());
    assert!(rules_of_the_run.is_spec_rule("myext-10101"));
    assert!(!rules_of_the_run.is_spec_rule("myext-empty-list"));

    // 5. The document is written to a file and read back: the declaration the editing cast
    //    introduced, the package attribute and the package element all survive the round trip.
    let path = std::env::temp_dir().join("biodivine-lib-sbml-extension-example.xml");
    let path = path.to_str().expect("The temporary path is valid UTF-8.");
    document.write_path(path)?;
    let read_back = Sbml::read_path(path)?;
    std::fs::remove_file(path)?;

    let model = read_back
        .model()
        .get()
        .expect("The written document contains a model.");
    let view = model.cast::<MyExtensionModel>()?;
    assert_eq!(view.strictness().get(), Some(0.75));
    assert_eq!(
        view.tuning_parameters().get().map(|it| it.len()),
        Some(1),
        "the package's list survives the round trip"
    );

    println!(
        "The extension is declared in the written document, `myext:strictness` is {:.2} and the \
         package's list holds {} element(s). The package's rules report {} issue(s) on the broken \
         document and none on the good one.",
        view.strictness().get().unwrap_or_default(),
        view.tuning_parameters()
            .get()
            .map(|it| it.len())
            .unwrap_or(0),
        broken
            .validate_with(&[&rules])?
            .iter()
            .filter(|it| it.rule().starts_with("myext-"))
            .count()
    );
    Ok(())
}

/// The entry point of the example; `tests/extension-example.rs` calls it as well, so that the code
/// the book includes is the code the test runs.
pub fn main() {
    run().expect("The example runs on the documents it builds.");
}
