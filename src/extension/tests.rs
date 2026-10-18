//! Unit tests of the extension surface: the cast and the injection of a package's rules.

use crate::core::Model;
use crate::extension::{ExtensionCast, ExtensionError, ExtensionOf, ExtensionValidation};
use crate::xml::{
    Document, OptionalSbmlChild, OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty,
    XmlChildDefault, XmlElement, XmlList, XmlWrapper,
};
use crate::{
    CancellationCheck, RuleDescriptor, RuleKind, RuleSet, Sbml, SbmlError, SbmlIssue,
    SbmlIssueSeverity, SbmlResult, FBC_NAMESPACE_URLS, NS_FBC, URL_SBML_CORE,
};

/// The namespace of the test package: the URL has the shape of a namespace of the standard, so that
/// the content of the package is not an "undefined SBML namespace" (rule 10102).
///
/// The element type a package adds is declared with `#[derive(XmlWrapper)]`; that declaration is
/// exercised outside this crate (`tests/extension_object.rs`) and by the worked example
/// (`examples/extension-example.rs`), while the tests here need only the package's *view* of a core
/// object, so the items of the list below are raw elements.
const MY_EXTENSION: (&str, &str) = (
    "myext",
    "http://www.sbml.org/sbml/level3/version2/myext/version1",
);

/// The view the test package adds to a `Model`.
struct MyExtensionModel {
    model: Model,
}

impl ExtensionOf<Model> for MyExtensionModel {
    const PACKAGE: (&'static str, &'static str) = MY_EXTENSION;

    fn from_object(model: Model) -> Self {
        MyExtensionModel { model }
    }

    fn object(&self) -> &Model {
        &self.model
    }
}

impl MyExtensionModel {
    /// The attribute the package adds to `<model>`.
    fn strictness(&self) -> OptionalSbmlProperty<'_, f64> {
        self.property("strictness")
    }

    /// The list of elements the package adds to `<model>`.
    fn parameters(&self) -> OptionalSbmlChild<'_, XmlList<XmlElement>> {
        self.child("listOfMyParameters")
    }
}

/// A view whose package is one this crate knows, to pin the version-set half of the declaration
/// check.
struct KnownPackageModel {
    model: Model,
}

impl ExtensionOf<Model> for KnownPackageModel {
    const PACKAGE: (&'static str, &'static str) = NS_FBC;

    fn from_object(model: Model) -> Self {
        KnownPackageModel { model }
    }

    fn object(&self) -> &Model {
        &self.model
    }
}

/// The rules of the test package: one rule of its own specification and one library lint.
struct MyExtensionRules {
    rules: RuleSet,
}

impl MyExtensionRules {
    fn new() -> MyExtensionRules {
        MyExtensionRules {
            rules: RuleSet::new(vec![
                RuleDescriptor::new(
                    "myext-10101",
                    RuleKind::SpecRule,
                    SbmlIssueSeverity::Error,
                    "The package requires a strictness between 0 and 1.",
                ),
                RuleDescriptor::new(
                    "myext-empty",
                    RuleKind::LibraryLint,
                    SbmlIssueSeverity::Info,
                    "The package's list of parameters is present but empty.",
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
        check.tick()?;
        let Some(model) = document.model().get() else {
            return Ok(());
        };
        let Ok(view) = model.cast::<MyExtensionModel>() else {
            return Ok(());
        };
        if let Some(strictness) = view.strictness().get() {
            if !(0.0..=1.0).contains(&strictness) {
                issues.push(SbmlIssue::new_in(
                    self.rules(),
                    "myext-10101",
                    &model,
                    format!(
                        "The strictness of this model is {strictness}, which is outside [0, 1]."
                    ),
                ));
            }
        }
        if let Some(parameters) = view.parameters().get() {
            if parameters.is_empty() {
                issues.push(SbmlIssue::new_in(
                    self.rules(),
                    "myext-empty",
                    &model,
                    "The model declares the package's list of parameters and leaves it empty.",
                ));
            }
        }
        Ok(())
    }
}

/// A document that declares nothing but the core namespace.
fn plain_document() -> Sbml {
    Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2"><model/></sbml>"#
    ))
    .expect("The test document is well formed.")
}

/// The model of a document, which every test document here has.
fn model_of(document: &Sbml) -> Model {
    document
        .model()
        .get()
        .expect("The test document contains a model.")
}

/// A typed object that is not part of an SBML document: a wrapper around an element of a document
/// whose root is not an `<sbml>` element.
fn object_outside_an_sbml_document() -> Model {
    let document =
        Document::parse_string(r#"<?xml version="1.0"?><probe/>"#).expect("Well-formed XML.");
    Model::from(XmlElement::new(
        document.root().expect("The probe has a root."),
    ))
}

#[test]
fn the_read_cast_needs_the_declaration_and_the_editing_cast_creates_it() {
    let document = plain_document();
    let model = model_of(&document);

    let error = model
        .cast::<MyExtensionModel>()
        .err()
        .expect("The document declares no package, so the read cast cannot succeed.");
    assert_eq!(
        error,
        ExtensionError::NotDeclared {
            package: MY_EXTENSION.1
        }
    );
    assert!(error.is_not_declared());
    assert_eq!(error.package(), MY_EXTENSION.1);

    // The editing cast declares the package and produces the view.
    let view = model
        .cast_or_activate::<MyExtensionModel>()
        .expect("The editing cast declares the package.");
    assert_eq!(view.object().xml_element().tag_name(), "model");
    assert_eq!(
        document.declared_package_prefix(MY_EXTENSION),
        Ok("myext".to_string())
    );

    // ... and the read cast now succeeds, because the document is declared. A second editing cast
    // changes nothing.
    assert!(model.cast::<MyExtensionModel>().is_ok());
    document
        .activate_package(MY_EXTENSION)
        .expect("Activating a declared package is a no-op.");
    assert!(model.cast::<MyExtensionModel>().is_ok());
    assert_eq!(
        document.xml_element().get_attribute("required").as_deref(),
        Some("false"),
        "the editing cast declares the package as not required"
    );
}

#[test]
fn a_document_that_binds_the_package_url_under_another_prefix_is_declared() {
    // The declaration is looked up by URL: the prefix is only the spelling this crate writes with.
    let document = Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:other="{}" other:required="false"
                level="3" version="2"><model/></sbml>"#,
        MY_EXTENSION.1
    ))
    .expect("The test document is well formed.");
    let model = model_of(&document);

    assert!(model.cast::<MyExtensionModel>().is_ok());
    assert_eq!(
        document.declared_package_prefix(MY_EXTENSION),
        Ok("other".to_string())
    );
}

#[test]
fn a_published_version_url_of_a_known_package_counts_as_declared() {
    // A package this crate knows accepts every published URL of it, so a document that binds
    // Version 1 of FBC declares the FBC package (the crate writes Version 2).
    let document = Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:fbc="{}" fbc:required="false"
                level="3" version="1"><model/></sbml>"#,
        FBC_NAMESPACE_URLS[0]
    ))
    .expect("The test document is well formed.");
    let model = model_of(&document);

    assert_ne!(FBC_NAMESPACE_URLS[0], NS_FBC.1);
    assert!(model.cast::<KnownPackageModel>().is_ok());
}

#[test]
fn a_half_declaration_is_not_a_declaration_and_the_editing_cast_completes_it() {
    // A namespace declaration without the `required` attribute is not a package declaration: rule
    // 10102 is about the *element* namespace, while the declaration of a package is the pair.
    let document = Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:myext="{}"
                level="3" version="2"><model/></sbml>"#,
        MY_EXTENSION.1
    ))
    .expect("The test document is well formed.");
    let model = model_of(&document);

    assert!(model.cast::<MyExtensionModel>().is_err());
    model
        .cast_or_activate::<MyExtensionModel>()
        .expect("The editing cast completes the declaration.");
    assert_eq!(
        document.declared_package_prefix(MY_EXTENSION),
        Ok("myext".to_string())
    );
    assert!(model.cast::<MyExtensionModel>().is_ok());
}

#[test]
fn an_object_outside_an_sbml_document_cannot_be_cast() {
    let model = object_outside_an_sbml_document();
    let error = model
        .cast::<MyExtensionModel>()
        .err()
        .expect("A probe document is not an SBML document.");
    assert_eq!(
        error,
        ExtensionError::NotInDocument {
            package: MY_EXTENSION.1
        }
    );

    let error = model
        .cast_or_activate::<MyExtensionModel>()
        .err()
        .expect("There is no `<sbml>` element to declare the package on.");
    assert_eq!(
        error,
        ExtensionError::NotInDocument {
            package: MY_EXTENSION.1
        }
    );
}

#[test]
fn the_view_reads_and_writes_the_package_data() {
    let document = plain_document();
    let model = model_of(&document);
    let view = model
        .cast_or_activate::<MyExtensionModel>()
        .expect("The editing cast declares the package.");

    // The package attribute of `<model>`.
    assert_eq!(view.strictness().get(), None);
    view.strictness().set_some(&0.5);
    assert_eq!(view.strictness().get(), Some(0.5));

    // The package's own element, written in the package namespace and added to its list. The
    // package namespace is declared on the element when the element is created, so the written
    // document uses the package prefix the declaration names.
    let parameter = XmlElement::new_qualified(
        document.xml_element().document(),
        "myParameter",
        MY_EXTENSION,
    );
    let parameters = view.parameters().get_or_create();
    parameters.push(parameter);
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters.get(0).namespace_url(), MY_EXTENSION.1);

    // The values survive a serialization round trip of the document.
    let text = document
        .to_xml_string()
        .expect("The document can be written.");
    let read_back = Sbml::read_str(&text).expect("The document can be read back.");
    let view = model_of(&read_back)
        .cast::<MyExtensionModel>()
        .expect("The package is declared in the document that was written.");
    assert_eq!(view.strictness().get(), Some(0.5));
    assert_eq!(view.parameters().get().map(|it| it.len()), Some(1));
}

#[test]
fn an_injected_rule_reports_with_its_own_id_kind_and_severity() {
    let document = Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:myext="{}" myext:required="false"
                level="3" version="2">
             <model myext:strictness="2.0"/>
           </sbml>"#,
        MY_EXTENSION.1
    ))
    .expect("The test document is well formed.");
    let model = model_of(&document);

    let extension = MyExtensionRules::new();
    let issues = document
        .validate_with(&[&extension])
        .expect("No cancellation trigger is registered.");
    let issue = issues
        .iter()
        .find(|it| it.rule() == "myext-10101")
        .expect("The package's rule reports the strictness outside [0, 1].");
    assert_eq!(issue.kind(), RuleKind::SpecRule);
    assert_eq!(issue.severity(), SbmlIssueSeverity::Error);
    assert!(issue.message().contains("2"));

    // The lint of the package fires on the same document, with its own kind and severity.
    let parameters = model
        .cast::<MyExtensionModel>()
        .expect("The package is declared.")
        .parameters()
        .get_or_create();
    assert!(parameters.is_empty());
    let issues = document
        .validate_with(&[&extension])
        .expect("Not cancelled.");
    let lint = issues
        .iter()
        .find(|it| it.rule() == "myext-empty")
        .expect("The package's lint reports the empty list.");
    assert_eq!(lint.kind(), RuleKind::LibraryLint);
    assert_eq!(lint.severity(), SbmlIssueSeverity::Info);
}

#[test]
fn validate_alone_does_not_run_the_rules_of_an_extension() {
    let document = Sbml::read_str(&format!(
        r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2"><model/></sbml>"#
    ))
    .expect("The test document is well formed.");
    let issues = document.validate().expect("Not cancelled.");
    assert!(issues.iter().all(|it| !it.rule().starts_with("myext-")));
}

#[test]
fn the_rule_set_of_a_run_classifies_injected_rules() {
    let extension = MyExtensionRules::new();
    let rules = RuleSet::CORE.with(extension.rules());

    // The classification is the property of the *rule*, and the census of the test-suite harness
    // asks about it through exactly this predicate.
    assert!(rules.is_spec_rule("myext-10101"));
    assert!(!rules.is_spec_rule("myext-empty"));
    // The table of the crate alone cannot tell the package's lint apart from a specification rule:
    // an id no descriptor describes counts as a specification rule, because the harness asks about
    // the ids of the suite's own expectation files. A run that injects a package therefore has to
    // classify with the combined set, which is what this test pins.
    assert!(RuleSet::CORE.is_spec_rule("myext-10101"));
    assert!(RuleSet::CORE.is_spec_rule("myext-empty"));
    // The crate's own rules keep their classification in the combined set.
    assert!(rules.is_spec_rule("10301"));
    assert!(!rules.is_spec_rule("unused-namespace"));
    // An id the set does not know at all stays a specification rule, because the harness asks about
    // the ids of the suite's own expectations.
    assert!(rules.is_spec_rule("80701"));
}

/// An extension whose check cancels the validation it is part of.
struct CancellingExtension {
    rules: RuleSet,
    trigger: cancel_this::CancelAtomic,
    ran: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ExtensionValidation for CancellingExtension {
    fn rules(&self) -> &RuleSet {
        &self.rules
    }

    fn validate(
        &self,
        _document: &Sbml,
        _issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        self.ran.store(true, std::sync::atomic::Ordering::SeqCst);
        // The package asks for the operation to stop, and the countdown it was handed is the one of
        // the whole call, so its own next tick reports the cancellation.
        self.trigger.cancel();
        for _ in 0..=(crate::cancellation::CHECK_INTERVAL + 1) {
            check.tick()?;
        }
        Ok(())
    }
}

#[test]
fn a_trigger_that_fires_inside_an_extension_check_stops_the_validation() {
    let trigger = cancel_this::CancelAtomic::new();
    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let extension = CancellingExtension {
        rules: RuleSet::new(vec![]),
        trigger: trigger.clone(),
        ran: ran.clone(),
    };
    let document = plain_document();

    // The built-in phases do not cancel anything; the package's own check does, and the countdown
    // it shares with the call turns that into the cancellation of the whole operation.
    let result: Result<Vec<SbmlIssue>, SbmlError> =
        cancel_this::on_atomic(trigger, || document.validate_with(&[&extension]));
    assert!(matches!(result, Err(SbmlError::Cancelled(_))));
    assert!(ran.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn an_already_fired_trigger_aborts_before_the_extension_runs() {
    let trigger = cancel_this::CancelAtomic::new();
    trigger.cancel();
    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let extension = CancellingExtension {
        rules: RuleSet::new(vec![]),
        trigger: trigger.clone(),
        ran: ran.clone(),
    };
    let document = plain_document();

    let result: Result<Vec<SbmlIssue>, SbmlError> =
        cancel_this::on_atomic(trigger, || document.validate_with(&[&extension]));
    assert!(matches!(result, Err(SbmlError::Cancelled(_))));
    assert!(
        !ran.load(std::sync::atomic::Ordering::SeqCst),
        "a fired trigger aborts the call before the package's check starts"
    );
}
