//!
//! This crate provides a Rust interface for reading, editing, and validating Systems Biology
//! Markup Language (SBML) files. Main features:
//!
//!  - Complete support for the SBML Level 3 Version 2 core specification.
//!  - validation of the *required* SBML conformance rules, including validation of proper namespace usage.
//!  - Ability to (safely) edit invalid or partially corrupted files (e.g. to fix errors).
//!  - Full access to the underlying XML document through the [biodivine_lib_xml_dom] interface.
//!  - `Annotation`, `Notes` and other custom XML/HTML elements are fully accessible as raw `XmlElement` objects.
//!  - Unofficial or unsupported features can be accessed using `DynamicProperty` or `DynamicChild` wrappers.
//!
//! Reading a document preserves its text: the tree is a verbatim view of the source, so a read,
//! edit and write cycle keeps the content of `notes`, `annotation` and MathML elements intact.
//! Where SBML *interprets* text as a value (an identifier, a number or an enumerated value, in an
//! attribute or in a MathML element such as `ci`), the incidental whitespace around that value is
//! ignored, because the relevant XML Schema datatypes collapse it. See [`Sbml::read_str`].
//!
//! For basic usage, explore the documentation of SBML objects declared in the [core]
//! module. More advanced operations can be then performed using the XML abstraction layer
//! described in module [xml].
//!
//! Three pages explain the project rather than the API: [documentation::specification] maps the
//! SBML specification onto this crate (levels, versions, packages, the shape of a document),
//! [documentation::design] gives the reasoning behind the library, and
//! [documentation::rule_coverage] states which specification rules are implemented, per level and
//! package, with the gaps named.
//!
//! Note that primary documentation of the SBML objects is adapted from the SBML specification,
//! including figures.
//!
//! ### Example
//!
//! ```rust
//! use biodivine_lib_sbml::*;
//! use biodivine_lib_sbml::core::*;
//! use biodivine_lib_sbml::xml::*;
//!
//! let doc = Sbml::read_path("./test-inputs/model.sbml")
//!     .expect("This document is not valid XML.");
//!
//! // First, we want to know if the document we
//! // just read is valid SBML. Validating is a cancellable, fallible
//! // operation, so it returns a `Result`; here, no cancellation
//! // trigger is registered, and a document that breaks a rule is
//! // reported through the returned issues rather than as an error.
//! let issues = doc
//!     .validate()
//!     .expect("Validation cannot be cancelled here.");
//! if issues.len() > 0 {
//!     // Note that these could be just warnings/notes. //!
//!     // You can check `SbmlIssue::severity` of each item
//!     // to detect fatal errors.
//!     eprintln!("This document has issues:");
//!     for issue in issues {
//!         eprintln!("{:?}", issue);
//!     }
//! }
//!
//! let model = doc
//!     .model()
//!     .get()
//!     // This is strange but allowed by the specification.
//!     .expect("This document does not contain any model.");
//!
//! // Note that individual lists of model components
//! // are also optional in the specification. Here,
//! // an empty list is created if it does not exist.
//! let species = model.species().get_or_create();
//! let compartments = model.compartments().get_or_create();
//! println!(
//!     "This model has {} compartments and {} species.",
//!     compartments.len(),
//!     species.len(),
//! );
//!
//! // We can use `DynamicProperty` and `DynamicChild` to access
//! // items that are not in the SBML core specification.
//!
//! // For example, here, we are reading the list of qualitative species defined
//! // in the sbml-qual package as a "generic" list of `XmlElement` objects.
//! let qual_species: OptionalDynamicChild<XmlList<XmlElement>> =
//!     OptionalDynamicChild::new(model.xml_element(), "listOfQualitativeSpecies");
//! println!(
//!     "This model has {} qualitative species.",
//!     qual_species.get_or_create().len(),
//! );
//!
//! // We can also modify the model.
//!
//! // First, create a new instance of a `Species` object.
//! let species_id = SId::try_from("sp_1").unwrap();
//! let compartment_id = compartments.get(0).id().get();
//! let s = Species::new(model.xml_element().document(), &species_id, &compartment_id);
//! let species_name = "MySpecies".to_string();
//! s.name().set_some(&species_name);
//!
//! // Then, add it to the current list of species.
//! species.push(s);
//! assert_eq!(species.get(0).name().get(), Some(species_name));
//!
//! // Finally, we can print the model back as XML:
//! let xml_string = doc.to_xml_string()
//!     .expect("Encoding error.");
//!
//! println!("{} ... ", &xml_string[..200]);
//! ```
//!
//! # The SBML specification and its authors
//!
//! The object model this crate implements, the summaries of its rules and the figures of this
//! documentation are adapted from the SBML specification: *The Systems Biology Markup Language
//! (SBML): Language Specification for Level 3 Version 2 Core*, Release 2, 29 March 2019, by
//! Michael Hucka (Chair), Frank T. Bergmann, Claudine Chaouiya, Andreas Dräger, Stefan Hoops,
//! Sarah M. Keating, Matthias König, Nicolas Le Novère, Chris J. Myers, Brett G. Olivier,
//! Sven Sahle, James C. Schaff, Rahuman Sheriff, Lucian P. Smith, Dagmar Waltemath,
//! Darren J. Wilkinson and Fengkai Zhang - the SBML Editors, `sbml-editors@sbml.org`,
//! <https://sbml.org>. The material is reused with their permission; `README.md` and
//! `docs-images/README.md` name the provenance of every figure.
//!

// The public surface of this crate is documented item by item, and the gate denies warnings, so an
// undocumented public item is a build failure rather than a hole in the reference.
#![warn(missing_docs)]

// The derive macros of `sbml-macros` generate absolute paths into this crate, so that a type
// declared *outside* it — an SBML extension, in particular — can use them as well: a macro that
// emitted `crate::..` would mean "the crate that invokes the macro", which is the wrong crate for
// an extension. The alias below is what makes the absolute path name this crate from within itself.
extern crate self as biodivine_lib_sbml;

use std::collections::HashSet;

use biodivine_lib_xml_dom::Document;
use embed_doc_image::embed_doc_image;

use xml::RequiredProperty;

use crate::constants::namespaces::Namespace;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_problems, CanTypeCheck};
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, Model, SBase, SId};
use crate::xml::{OptionalSbmlChild, OptionalXmlChild, SbmlObject, XmlElement, XmlWrapper};

/// The **FBC** (Flux Balance Constraints) package: [`GeneProduct`][constraint::GeneProduct],
/// [`Objective`][constraint::Objective], the flux bounds of a reaction and the gene-product
/// association of FBC Version 2.
pub mod constraint;
/// Defines [`Model`], [`Species`][core::Species], [`Compartment`][core::Compartment],
/// [`FunctionDefinition`][core::FunctionDefinition] and other data objects prescribed
/// by the SBML core specification.
pub mod core;

/// The documentation of the project rather than of the API: the specification this crate implements
/// ([`specification`](documentation::specification)), the decisions behind the library
/// ([`design`](documentation::design)) and which rules it covers
/// ([`rule_coverage`](documentation::rule_coverage), generated from the coverage matrix).
pub mod documentation;

/// The surface an SBML *package* written outside this crate uses: the cast of a core object to the
/// view the package adds ([`ExtensionOf`], [`ExtensionCast`]) and the injection of the package's
/// validation rules into a document's validation ([`ExtensionValidation`]).
pub mod extension;
/// The **Layout** package: [`Layout`][layout::Layout], the glyphs it draws and the curve segments
/// between them.
pub mod layout;

/// Defines [`Math`][math::Math], the typed layer over the MathML subset that SBML formulas use:
/// [`MathNode`][math::MathNode], the declared element types of the subset, and the closed operator
/// and constant sets.
pub mod math;
/// The **Qual** (Qualitative Models) package: [`QualitativeSpecies`][qual::QualitativeSpecies],
/// [`Transition`][qual::Transition] and their inputs, outputs and terms.
pub mod qual;

/// Defines [`Document`], [`XmlElement`], [`XmlWrapper`], [`XmlProperty`][xml::XmlProperty],
/// [`XmlChild`][xml::XmlChild] and other utility types or traits that can be used to safely
/// manipulate the underlying XML document.
pub mod xml;

/// The rules this crate reports: their ids, kinds, severities and summaries, and the
/// [`SbmlIssue`] a failed check produces.
mod validation;
pub use validation::{RuleDescriptor, RuleKind, RuleSet, SbmlIssue};

/// The cast of a core SBML object to the view an SBML package adds to it.
pub use crate::extension::ExtensionCast;
/// The error a cast to an extension view produces.
pub use crate::extension::ExtensionError;
/// The view an SBML package adds to a core SBML object.
pub use crate::extension::ExtensionOf;
/// The validation rules of an SBML package.
pub use crate::extension::ExtensionValidation;

/// The periodic cancellation check a long-running operation of this crate consults.
pub use crate::cancellation::CancellationCheck;

/// The SBML namespaces, as the `(prefix, url)` pairs this crate writes with, and the root
/// namespace set of the documents it recognizes.
///
/// A namespace is identified by its URL; the prefix is only the conventional spelling the crate
/// writes new names with, and a document that binds the URL under another prefix is read the same
/// way. An SBML extension needs three things from this list: [`NS_SBML_CORE`] (the namespace of the
/// elements it extends and of the attributes a core element carries), its own namespace (declared
/// with [`SbmlObject`] as a pair of its own, for example
/// `#[sbml_object(name = "myGraph", namespace = ("myext", "http://www.sbml.org/sbml/level3/version2/myext/version1"))]`), and
/// [`NS_XSI`] when one of its types is discriminated by `xsi:type` rather than by its local name.
///
/// The value type of these constants is [`Namespace`], which lives in the [`xml`]
/// module: this module exports the values, `xml` exports the type, so each item has exactly one
/// canonical path.
///
/// [`SBML_ROOT_NAMESPACES`] is the set of SBML document roots, which is data a later level module
/// extends; see its own documentation for exactly what a membership in that set means.
pub use crate::constants::namespaces::{
    is_package_namespace_of, package_of_namespace_url, package_urls_of, written_url_of,
    COMP_NAMESPACE_URLS, FBC_NAMESPACE_URLS, LAYOUT_NAMESPACE_URLS, NS_EMPTY, NS_FBC, NS_HTML,
    NS_LAYOUT, NS_MATHML, NS_QUAL, NS_SBML_CORE, NS_XSI, QUAL_NAMESPACE_URLS, SBML_ROOT_NAMESPACES,
    URL_EMPTY, URL_HTML, URL_MATHML, URL_PACKAGE_FBC, URL_PACKAGE_LAYOUT, URL_PACKAGE_QUAL,
    URL_SBML_CORE, URL_XSI,
};

/// Writes the `xsi:type` attribute of an element, declaring the XML Schema instance namespace on it
/// when necessary.
///
/// This is the write half of [`SbmlObject::XSI_TYPE`], and the constructor
/// an extension's `#[sbml_object(..)]` declaration generates calls it; it is public because an
/// extension that sets the attribute outside a constructor needs the same declaration.
pub use crate::xml::set_xsi_type;

/// **(internal)** The periodic cancellation check shared by every document walk of this crate.
pub(crate) mod cancellation;
/// **(internal)** An internal module which defines constant values relevant for SBML, such as
/// namespace URLs or mappings assigning elements their allowed attributes.
pub(crate) mod constants;
/// **(test)** A helper module for executing the syntactic SBML test suite as part of the
/// standard unit tests.
#[cfg(test)]
pub mod test_suite;

/// The error type produced by the fallible operations of this crate.
///
/// Parsing, reading and writing an SBML document can fail for several reasons, and this type
/// keeps them apart. Most variants wrap an underlying error; [`SbmlError::Xml`] covers
/// well-formedness, namespace and serialization problems detected by the XML layer, and
/// [`SbmlError::Cancelled`] is returned when a [`cancel_this`] trigger fires during a
/// long-running operation.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SbmlError {
    /// The document could not be parsed or serialized as XML.
    #[error("XML error: {0}")]
    Xml(#[from] biodivine_lib_xml_dom::XmlError),
    /// A cancellation trigger fired while an SBML operation was running.
    #[error("The operation was cancelled.")]
    Cancelled(#[from] cancel_this::Cancelled),
    /// The parsed document has no root element.
    ///
    /// This is a defensive variant: the XML parser already rejects a document without a root
    /// element. A document whose root element is not `<sbml>` parses successfully and is reported
    /// by [`Sbml::validate`] as rule 10102 instead of failing the read.
    #[error("The document does not contain a root element.")]
    NotSbmlDocument,
    /// The document declares an SBML level/version outside the range this crate can process.
    ///
    /// The document declares an SBML level and version that this crate does not model.
    ///
    /// Returned by level/version dispatch when a document declares a level/version that the crate
    /// does not support.
    #[error("Unsupported SBML level {level} version {version}.")]
    UnsupportedLevel {
        /// The level the document declares.
        level: u32,
        /// The version the document declares.
        version: u32,
    },
    /// An I/O error occurred while reading or writing a document.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A [`Result`] whose error is an [`SbmlError`].
pub type SbmlResult<T> = Result<T, SbmlError>;

/// The version of this crate, as its `Cargo.toml` declares it.
///
/// It is exposed so that a consumer which reports it (the Python bindings report it as
/// `biodivine.sbml.__version__`) reads the number from the crate itself instead of repeating it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convert an error from the underlying XML layer into an [`SbmlError`].
///
/// Cancellation is pulled out into its own variant so that a caller can distinguish "the
/// operation was aborted on request" from "the document is not valid XML".
fn sbml_error(error: biodivine_lib_xml_dom::XmlError) -> SbmlError {
    match error {
        biodivine_lib_xml_dom::XmlError::Cancelled(inner) => SbmlError::Cancelled(inner),
        other => SbmlError::Xml(other),
    }
}

/// The SBML container object
/// (Section 4.1; [specification](https://raw.githubusercontent.com/combine-org/combine-specifications/main/specifications/files/sbml.level-3.version-2.core.release-2.pdf)).
///
/// ## 4.1 The SBML Container
///
/// <!-- A minor hack to position the UML figure nicely in the page. -->
/// <style>
/// img[alt=UML] {
///     width: 80%;
///     display: block;
///     margin: 0 auto;
///     margin-top: 1rem;
///     margin-bottom: 1rem;
/// }
/// </style>
///
/// ![UML][sbml-container]
///
/// Following the XML declaration, the outermost portion of a model expressed in Level 3 consists
/// of an object of class [`Sbml`]. This class contains three required attributes
/// (*level*, *version* and *xmlns*), and an optional *model* element.
///
/// The [`Sbml`] class defines the structure and content of the `sbml` outermost element in an SBML
/// file. The following is an abbreviated example of [`Sbml`] class object translated into XML
/// form for an SBML Level 3 Version 2 Core document. Here, ellipses (“...”) are used to indicate
/// content elided from this example:
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
///     <sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
///      ...
///         <model ...> ... </model>
///     </sbml>
/// ```
///
/// The attribute `xmlns` declares the XML namespace used within the `sbml` element. The URI for
/// SBML Level 3 Version 2 Core is `http://www.sbml.org/sbml/level3/version2/core`. All SBML Level
/// 3 Version 2 Core elements and attributes must be placed in this namespace either by assigning
/// the default namespace as shown in the example above, or using a tag prefix on every element.
/// The `sbml` element may contain additional attributes, in particular, attributes to support the
/// inclusion of SBML Level 3 packages; see Section 4.1.3. For purposes of checking conformance
/// to the SBML Level 3 Core specification, only the elements and attributes in the SBML Level
/// 3 Core XML namespace are considered.
///
/// ### 4.1.1 The `id` and `name` attributes
/// Because SBML inherits from [`SBase`], it has optional `id`, `name`, `sboTerm` and `metaid`
/// attributes. SBML Level 3 Version 2 Core does not define a purpose for these attributes;
/// moreover, being outside the [`Model`] namespace, the `id` attribute is not subject to the
/// uniqueness constraints of `SId` values inside [`Model`] objects.
///
/// ### 4.1.2 The `model` element
/// The actual model contained within an SBML document is defined by an instance of the [`Model`]
/// class element. The structure of this object and its use are described in Section 4.2.
/// An SBML document may contain at most one model definition.
///
#[embed_doc_image("sbml-container", "docs-images/uml-sbml-container.svg")]
#[derive(Clone, Debug)]
pub struct Sbml {
    xml: Document,
    sbml_root: XmlElement,
}

/// The SBML-defined components of the [`Sbml`] container class.
impl Sbml {
    /// The `<sbml>` element this container describes.
    ///
    /// The container is a document plus the root element of that document; this accessor is the
    /// escape hatch between the two, for a caller that needs the XML view of a document (its
    /// children, its attributes, its text, or the document it belongs to).
    pub fn root(&self) -> &XmlElement {
        &self.sbml_root
    }

    /// The `model` child of the `<sbml>` container, if the document has one.
    pub fn model(&self) -> OptionalSbmlChild<'_, Model> {
        OptionalSbmlChild::new_object_model(&self.sbml_root, "model", NS_SBML_CORE)
    }

    /// The `level` attribute of the `<sbml>` container.
    pub fn level(&self) -> RequiredProperty<'_, u32> {
        RequiredProperty::new(&self.sbml_root, "level")
    }

    /// The `version` attribute of the `<sbml>` container.
    pub fn version(&self) -> RequiredProperty<'_, u32> {
        RequiredProperty::new(&self.sbml_root, "version")
    }
}

/// Other methods for creating and manipulating [`Sbml`] container.
impl Sbml {
    /// Try to create a new instance of [`Sbml`] for the given instance of an [`XmlWrapper`]
    /// (transitive) child element.
    ///
    /// This method can fail (return `None`) when the element is not a member of a valid SBML
    /// document (i.e. the root is not an `sbml` tag).
    ///
    pub fn try_for_child<T: XmlWrapper>(child: &T) -> Option<Sbml> {
        Self::try_for_child_element(child.xml_element())
    }

    /// Same as [Self::try_for_child], but uses the raw [XmlElement], which is a bit more flexible.
    pub fn try_for_child_element(child: &XmlElement) -> Option<Sbml> {
        let root = child.document_root()?;
        if root.tag_name().as_str() == "sbml" {
            Some(Sbml {
                xml: root.document(),
                sbml_root: root,
            })
        } else {
            None
        }
    }

    /// Parse a document from a string.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Xml`] if `file_contents` is not a well-formed XML 1.0 document
    /// with a single root element, or if it declares an encoding other than UTF-8. Returns
    /// [`SbmlError::Cancelled`] if a cancellation trigger fires while the document is being
    /// parsed.
    pub fn read_str(file_contents: &str) -> SbmlResult<Sbml> {
        let doc = Document::parse_string(file_contents).map_err(sbml_error)?;
        let root = doc.root().ok_or(SbmlError::NotSbmlDocument)?;
        Ok(Sbml {
            xml: doc,
            sbml_root: XmlElement::new(root),
        })
    }

    /// Read and parse a document from a file.
    ///
    /// # Errors
    ///
    /// As [`Sbml::read_str`], plus [`SbmlError::Io`] if the file cannot be read.
    pub fn read_path(path: &str) -> SbmlResult<Sbml> {
        let file_contents = std::fs::read_to_string(path)?;
        Self::read_str(file_contents.as_str())
    }

    /// Serialize this document into a file.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Xml`] if the document cannot be serialized, and [`SbmlError::Io`]
    /// if the file cannot be written.
    pub fn write_path(&self, path: &str) -> SbmlResult<()> {
        self.xml.write_file(path).map_err(sbml_error)?;
        Ok(())
    }

    /// Serialize this document into a string.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Xml`] if the document cannot be serialized.
    pub fn to_xml_string(&self) -> SbmlResult<String> {
        self.xml.write_string().map_err(sbml_error)
    }

    /// Perform a basic type checking procedure, one check per element of the document.
    ///
    /// The type check is the oracle of the typed API: it decides whether the *typed reads* of an
    /// element are usable. A type-check issue on an element means that reading one of its properties
    /// or children through the typed accessors can panic (an attribute value that does not parse,
    /// a required child that is not there), which is why [Sbml::validate] consults the same checks
    /// before it walks an object (see [Sbml::validate] for the contract, and
    /// [`SbmlValidable`](crate::validation) for the guard itself).
    ///
    /// The pass accounts for every element it visits, so a cancellation request that arrives
    /// while it is running stops it.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`] if a cancellation trigger of the current thread has fired.
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        // Note: a document that contains more than one root element, or no root element at all,
        // is rejected while parsing, so the previous explicit multi-root check is unnecessary.
        // Only a root element that is not `<sbml>` can still reach this point.

        let root_element = self.sbml_root.xml_element();
        if root_element.tag_name() != "sbml" {
            let message = format!("Invalid root element <{}> found.", root_element.tag_name());
            issues.push(SbmlIssue::new("10102", &self.sbml_root, message));
        }

        issues.extend(type_check_problems(&self.sbml_root));

        // Rule 20101: the `<sbml>` container must declare the XML namespace of SBML, and the
        // declaration must be consistent with the `level` and `version` attributes. The condition
        // checked here is the first half - the *root element's own* namespace has to be a published
        // SBML namespace - which is the half that is a property of the root alone. It is
        // deliberately about the element's namespace URL and not about the presence of an `xmlns`
        // declaration: a document that binds the SBML namespace under a prefix and writes
        // `<a:sbml>` is valid, while one whose `<sbml>` element is in no namespace (or in a foreign
        // one, or in an unpublished variant such as `.../level2/version6`) is not.
        //
        // This is the same condition that decides whether the object model resolves at all (see
        // `constants::version`), so the rule and the model agree by construction rather than by two
        // implementations that can drift apart.
        if crate::constants::version::root_is_sbml_element(&self.sbml_root)
            && !crate::constants::version::root_is_published_sbml_root(&self.sbml_root)
        {
            let namespace = self.sbml_root.xml_element().namespace_url();
            let message = format!(
                "The <sbml> container element must declare the XML namespace for SBML, and this \
                 declaration must be consistent with the values of the 'level' and 'version' \
                 attributes on the <sbml> element. The namespace of the root element is \
                 '{namespace}', which no SBML specification defines."
            );
            issues.push(SbmlIssue::new("20101", &self.sbml_root, message));
        }

        // Rules 20102 and 20103: the declared `level` and `version` must be consistent with the
        // namespace the root declares. The two rules are the attribute halves of 20101, and the
        // corpus's own documents pin both directions of each: a document that declares a Level 1
        // level under a Level 2 namespace reports 20102 and not 20103 (`20102-fail-01-01-sev2-l1v1`),
        // while one that declares a Level 2 version under the Level 2 Version 1 namespace reports
        // 20103 and not 20102 (`20103-fail-01-01-sev2-l2v2`).
        //
        // A *missing* or unparsable attribute is reported by the type check of the root itself (the
        // required-attribute guard maps `level` to 20102 and `version` to 20103), so these two checks
        // only speak when the attribute has a value. They are also only meaningful when the namespace
        // is a published SBML namespace: an unpublished one is 20101's case, and asking it what
        // level it encodes would be reading a version out of a URL no specification defines.
        if crate::constants::version::root_is_published_sbml_root(&self.sbml_root) {
            let namespace = self.sbml_root.xml_element().namespace_url();
            let (namespace_level, namespace_version) =
                crate::constants::namespaces::level_version_of_namespace(&namespace);

            let declared_level = self
                .sbml_root
                .xml_element()
                .get_attribute("level")
                .and_then(|it| it.parse::<u32>().ok());
            if let Some(level) = declared_level {
                if level != namespace_level {
                    let message = format!(
                        "The value of the 'level' attribute of the <sbml> element must be \
                         consistent with the XML namespace of the document. The document declares \
                         Level {level}, while the namespace '{namespace}' is that of Level \
                         {namespace_level}."
                    );
                    issues.push(SbmlIssue::new("20102", &self.sbml_root, message));
                }
            }

            let declared_version = self
                .sbml_root
                .xml_element()
                .get_attribute("version")
                .and_then(|it| it.parse::<u32>().ok());
            if let Some(version) = declared_version {
                let consistent = match namespace_version {
                    // The namespace names one version, so that is the version the document is.
                    Some(expected) => version == expected,
                    // The namespace does not name a version (Level 1), so the question is whether the
                    // level has published the version at all.
                    None => crate::constants::version::published_versions(namespace_level)
                        .contains(&version),
                };
                if !consistent {
                    let message = format!(
                        "The value of the 'version' attribute of the <sbml> element must be \
                         consistent with the XML namespace of the document. The document declares \
                         Version {version}, while the namespace '{namespace}' is that of \
                         Level {namespace_level}{}.",
                        match namespace_version {
                            Some(expected) => format!(" Version {expected}"),
                            None => " (which does not name a version)".to_string(),
                        }
                    );
                    issues.push(SbmlIssue::new("20103", &self.sbml_root, message));
                }
            }
        }

        if let Some(model) = self.model().get() {
            model.type_check(issues, check)?;
        }

        Ok(())
    }

    /// Validates the document against the validation rules specified in the
    /// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
    /// Eventual issues are returned in the vector. Empty vector represents a valid document.
    ///
    /// Each issue names the rule that found it ([`SbmlIssue::rule`]), the [kind](RuleKind) of that
    /// rule, the [severity](SbmlIssue::severity) the rule assigns to it, the element it was found
    /// on and a message that explains it:
    ///
    ///  * an issue of a [`RuleKind::SpecRule`] is a violation of a rule of the SBML standard, and
    ///    its id is the id the specification gives that rule (`10301`, `fbc-21207`, ...). Its
    ///    severity is `Error` for a rule the standard requires and `Warning` or `Info` for a rule
    ///    it recommends.
    ///  * an issue of a [`RuleKind::LibraryLint`] is a diagnostic of this library, not a rule of
    ///    the standard: the document is valid SBML without it. Its id is not a specification rule
    ///    id, and its severity is `Info` unless the diagnostic means that the typed view of the
    ///    document is unavailable ([`SbmlIssue::kind`] tells the two kinds apart).
    ///
    /// Both kinds are reported here, and the lints are reported for every document: they are
    /// appended after the type check, so a document whose typed view is unusable still reports its
    /// lints, while a fatal problem stays the first issue of the report. Only the specification
    /// rules are compared against the SBML test suite's expectations, because those list
    /// specification rule ids.
    ///
    /// The [rule table](RuleSet::CORE) is where a rule id acquires its kind and severity, so an
    /// issue cannot be reported with a severity that its rule does not have.
    ///
    /// ## Which documents resolve an object model
    ///
    /// The tables of the validation layer describe one level and version of SBML, and the object
    /// model resolves only for a document whose declared level and version this crate models. Three
    /// rules decide it, and the crate's `VALIDATION.md` states them with the tests that pin each of
    /// them:
    ///
    ///  1. the root element's **own namespace** has to be a published SBML root namespace
    ///     ([`SBML_ROOT_NAMESPACES`]); otherwise no model resolves and rule `20101` reports it;
    ///  2. the declared `level`/`version` **attributes** select the tables (the namespace is the
    ///     fallback when one of them is missing), because the specification makes the namespace a
    ///     redundant encoding of the same fact;
    ///  3. the pair has to be one the crate models; a published pair it does not model resolves no
    ///     model and is named by the `unmodelled-level` lint, so a caller is never told "no issues"
    ///     about a document that nothing checked.
    ///
    /// ```
    /// use biodivine_lib_sbml::Sbml;
    /// use biodivine_lib_sbml::xml::OptionalXmlChild;
    ///
    /// // A Level 3 Version 1 document resolves its model, and is checked against the Version 1
    /// // tables: `fast` is a legal reaction attribute there, so nothing is reported.
    /// let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3" version="1">
    ///     <model><listOfReactions><reaction id="r" reversible="true" fast="false"/></listOfReactions></model>
    /// </sbml>"#;
    /// let document = Sbml::read_str(source)?;
    /// assert!(document.model().get().is_some());
    /// assert!(document.validate()?.is_empty());
    ///
    /// // A document of a published level this crate does not model resolves no model, and the
    /// // `unmodelled-level` lint says so instead of reporting an empty result.
    /// let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level1" level="1" version="2"><model/></sbml>"#;
    /// let document = Sbml::read_str(source)?;
    /// assert!(document.model().get().is_none());
    /// let rules: Vec<String> = document.validate()?.into_iter().map(|it| it.rule().to_string()).collect();
    /// assert_eq!(rules, vec!["unmodelled-level".to_string()]);
    ///
    /// // A root whose namespace no specification defines is reported by rule 20101, and nothing
    /// // else is checked, because nothing in such a document can be interpreted.
    /// let source = r#"<sbml xmlns="http://example.org/not-sbml" level="3" version="2"><model/></sbml>"#;
    /// let document = Sbml::read_str(source)?;
    /// assert!(document.model().get().is_none());
    /// let rules: Vec<String> = document.validate()?.into_iter().map(|it| it.rule().to_string()).collect();
    /// assert_eq!(rules, vec!["20101".to_string()]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## What this call guarantees, and what a type-check issue means for the walk
    ///
    /// The call always produces a report; it never panics on a document, however broken. What it
    /// *reaches* is decided per object, in four steps:
    ///
    ///  1. the type check of the whole document, which reports one issue per element that is not
    ///     type-consistent;
    ///  2. the `unused-namespace` lint;
    ///  3. the walk of the object model, which visits every object whose own element is
    ///     type-consistent and, for the objects whose element is not, reports **one
    ///     `not-validated` issue** (a library lint of `Info` severity) that names the element and
    ///     the rule that rejected it, and does not descend into it;
    ///  4. nothing else: a type-check issue no longer suppresses the walk of the rest of the
    ///     document, so a document with one broken compartment is still validated everywhere else.
    ///
    /// So a caller detects that part of a document was *not* validated with
    /// [`SbmlIssue::is_not_validated`] (the rule id is `not-validated` and the kind is
    /// [`RuleKind::LibraryLint`]); the element the issue is attached to is the root of the subtree
    /// that was skipped, and its message names the first rule the type check reported for that
    /// element. A report without such an issue means that every object of the document was walked.
    ///
    /// **The typed accessors remain unsafe on an element with a type-check issue.** The type check
    /// is no longer a gate for the walk, but it is still the gate for typed access: reading a
    /// property or child of an element whose type check failed can panic. What the type check
    /// reports for an element is not enumerable from the outside - it emits `SANITY_CHECK`, the
    /// specification rules its element tables name (for example `10102`, `20517`) and the rules of
    /// the checks next to the element (for example `layout-10401`) - so the rule to follow is:
    /// treat the typed view of an element as unusable when the report names a problem on it, and
    /// look at [`SbmlIssue::is_not_validated`] for the subtrees the walk skipped. The evidence that
    /// this is sufficient is the corpus: the panic inventory of
    /// `scripts/corpus_panic_inventory.py` reports zero documents for the whole syntactic test
    /// suite, and the guard exists because it reported 73 before this design. To edit such a
    /// document, fix the reported problems (the crate allows editing a document that is invalid in
    /// other ways) and validate again before reading it through the typed API.
    ///
    /// ### Rule 10101
    /// is already satisfied implicitly by the use of the package *biodivine-lib-xml-dom* as writing
    /// is done only in UTF-8 and reading produces an error if the declared encoding is anything
    /// other than UTF-8. The specific error is covered in [Sbml::read_str].
    ///
    /// ### Rule 10104
    /// is already satisfied implicitly by the use of the package *biodivine-lib-xml-dom* as loading
    /// a document without an error ensures that the document conforms to the basic
    /// structural and syntactic constraints.
    ///
    /// ## Cancellation
    ///
    /// The whole call is cancellable, through the same mechanism as reading and writing: a
    /// [`cancel_this`] trigger registered on the current thread aborts the call, and the caller
    /// sees [`SbmlError::Cancelled`] instead of a report. There is no separate parameter for it,
    /// because a trigger registered for a long-running operation is expected to cover every step
    /// of that operation.
    ///
    /// The four phases are all covered:
    ///
    ///  1. the very first step of the call consults the triggers, so a trigger that has already
    ///     fired aborts the call before any work is done — in particular before the type check and
    ///     before the lints below, so an already-fired trigger produces `Cancelled` and no report
    ///     at all;
    ///  2. the type check of the document, one step per element;
    ///  3. the `unused-namespace` lint pass, one step per element;
    ///  4. the rule-by-rule walk of the object model, one step per object (the guard of the walk
    ///     accounts for the object before the object's own checks do).
    ///
    /// The trigger is consulted once every 1024 steps rather than on every step, and every phase
    /// shares the same countdown, so a caller that cancels a large document is delayed by at most a
    /// few hundred steps of whichever phase is running. See the crate's `cancellation` module for
    /// the policy and for its mirror in the XML layer, which covers the read and write halves of an
    /// operation.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`] if a cancellation trigger registered on the current thread
    /// has fired. A document that violates rules of the specification is *not* an error: it is
    /// reported through the returned issues.
    ///
    /// ## SBML packages
    ///
    /// This method validates the rules of this crate. The rules of an SBML *package* are the
    /// package's own, and they take part in the validation of a document through
    /// [`Sbml::validate_with`], which runs the checks of every
    /// [`ExtensionValidation`] it is given after the checks below:
    ///
    /// ```
    /// # use biodivine_lib_sbml::extension::ExtensionValidation;
    /// # use biodivine_lib_sbml::CancellationCheck;
    /// # use biodivine_lib_sbml::xml::{Document, OptionalXmlChild, XmlWrapper};
    /// # use biodivine_lib_sbml::{
    /// #     RuleDescriptor, RuleKind, RuleSet, Sbml, SbmlIssue, SbmlIssueSeverity, SbmlResult,
    /// #     URL_SBML_CORE,
    /// # };
    /// # struct MyExtensionRules { rules: RuleSet }
    /// # impl MyExtensionRules {
    /// #     fn new() -> Self { MyExtensionRules { rules: RuleSet::new(vec![RuleDescriptor::new(
    /// #         "myext-10101", RuleKind::SpecRule, SbmlIssueSeverity::Error, "A model must not be empty.")]) } }
    /// # }
    /// # impl ExtensionValidation for MyExtensionRules {
    /// #     fn rules(&self) -> &RuleSet { &self.rules }
    /// #     fn validate(&self, document: &Sbml, issues: &mut Vec<SbmlIssue>, _check: &mut CancellationCheck) -> SbmlResult<()> {
    /// #         if let Some(model) = document.model().get() {
    /// #             if model.xml_element().child_elements().is_empty() {
    /// #                 issues.push(SbmlIssue::new_in(self.rules(), "myext-10101", &model, "Empty model."));
    /// #             }
    /// #         }
    /// #         Ok(())
    /// #     }
    /// # }
    /// let document = Sbml::read_str(&format!(
    ///     r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2"><model/></sbml>"#
    /// ))?;
    /// let extension = MyExtensionRules::new();
    ///
    /// // The package's rule is not part of this method's report ...
    /// assert!(document.validate()?.iter().all(|it| it.rule() != "myext-10101"));
    ///
    /// // ... and runs when the package is passed to `validate_with`.
    /// let issues = document.validate_with(&[&extension])?;
    /// assert!(issues.iter().any(|it| it.rule() == "myext-10101"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// So a caller that wants the rules of a package to run passes the package explicitly. This is
    /// a property of the call rather than of the document on purpose: the report of a validation is
    /// then a function of the document and of the extensions the caller asked about, and not of a
    /// process-wide registry (two documents in one process, or the SBML test-suite harness, would
    /// otherwise disagree about what "the rules" are). A package that is relevant only for the
    /// documents that *declare* it asks
    /// [`declared_package_prefix`](Sbml::declared_package_prefix) itself, which is also what
    /// [`ExtensionCast`] does.
    pub fn validate(&self) -> SbmlResult<Vec<SbmlIssue>> {
        self.validate_with(&[])
    }

    /// Validates the document as [`Sbml::validate`] and then runs the checks of the SBML packages in
    /// `extensions`, in order, adding their issues to the same report.
    ///
    /// The result is the report of the whole document: the issues of this crate's rules (all of
    /// which are checked, exactly as [`Sbml::validate`] does) followed by the issues the package
    /// checks pushed. Each issue names its own rule, and the id, the kind and the severity of a rule
    /// of a package are the ones its
    /// [`RuleDescriptor`] gives it - a package rule is reported through
    /// [`SbmlIssue::new_in`] with the package's [rule set](crate::ExtensionValidation::rules), which
    /// is what makes an injected rule indistinguishable from a built-in one at the report.
    ///
    /// The packages are described by [`ExtensionValidation`], which is
    /// also where the contract of a check (the order of the phases, the shared cancellation
    /// countdown, the documents a check can be called on) is written down. A package's checks are
    /// called for **every** document, including the ones that do not declare the package, because
    /// whether the package's content is relevant is a question only the package can answer.
    ///
    /// ## Cancellation
    ///
    /// The extension checks are part of the same cancellable operation as the built-in phases: they
    /// receive the countdown of the call, so the walk of a package honours a request to stop, and a
    /// trigger that has fired aborts the call before the next package check starts. A cancelled call
    /// reports [`SbmlError::Cancelled`] instead of a partial report, exactly as [`Sbml::validate`]
    /// does.
    ///
    /// ## The rules of a validation run
    ///
    /// The rules a call compares against are the union of this crate's rules and the rules of the
    /// extensions, [`RuleSet::CORE`].`with`(extension's set). A caller that has to classify an id -
    /// the SBML test-suite harness asks whether a reported id is a specification rule, so that a
    /// library lint cannot turn a document the suite expects to pass into a mismatch - builds that
    /// union itself and asks it, in the same way the built-in run does.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`] if a cancellation trigger registered on the current thread
    /// has fired, including one that fires inside a package's check.
    pub fn validate_with(
        &self,
        extensions: &[&dyn ExtensionValidation],
    ) -> SbmlResult<Vec<SbmlIssue>> {
        let mut check = CancellationCheck::new();
        let mut issues: Vec<SbmlIssue> = vec![];

        // The first step of the call consults the triggers, before any phase does any work: a
        // trigger that has already fired aborts the call instead of producing a report.
        check.tick()?;

        self.type_check(&mut issues, &mut check)?;

        // The library lints are reported for every document: a document whose typed view is
        // unusable still has a dead namespace declaration to report, and a lint must not depend on
        // the type check's outcome. They are appended after the type check so that a fatal problem
        // stays the first issue of the report.
        validation::validate_unused_namespaces(&self.sbml_root, &mut issues, &mut check)?;
        validation::validate_package_rules(&self.sbml_root, &mut issues, &mut check)?;
        validation::validate_unmodelled_level(&self.sbml_root, &mut issues, &mut check)?;

        // The walk runs for every document. What a type-check problem means for it is decided per
        // object by the guard of `SbmlValidable::validate`: the object whose element is not
        // type-consistent is skipped and reported as `not-validated`, instead of a type-check
        // problem anywhere in the document suppressing the walk of the whole document.
        let mut identifiers: HashSet<SId> = HashSet::new();
        let mut meta_ids: HashSet<MetaId> = HashSet::new();

        validate_sbase(self, &mut issues, &mut identifiers, &mut meta_ids);

        if let Some(model) = self.model().get() {
            model.validate(&mut issues, &mut identifiers, &mut meta_ids, &mut check)?;
        }

        // The SBML packages take part after the walk of the object model: a package's check is
        // written against what this crate has already established about the document (the issues
        // above), and the countdown is shared, so a request to stop reaches the package's own walk
        // as well. A trigger that has already fired aborts before the next check starts.
        for extension in extensions {
            check.tick()?;
            extension.validate(self, &mut issues, &mut check)?;
        }

        Ok(issues)
    }
}

impl Default for Sbml {
    /// Creates a new blank SBML document with initial skeleton consisting of valid
    /// xml header and sbml root element.
    fn default() -> Self {
        let doc = Document::parse_string(constants::document::SBML_DEFAULT_DOCUMENT)
            .expect("The default SBML document is a valid XML document.");
        let root = doc
            .root()
            .expect("The default SBML document contains a root element.");
        Sbml {
            xml: doc,
            sbml_root: XmlElement::new(root),
        }
    }
}

impl XmlWrapper for Sbml {
    fn xml_element(&self) -> &XmlElement {
        &self.sbml_root
    }
}

impl From<Sbml> for XmlElement {
    fn from(value: Sbml) -> Self {
        value.sbml_root
    }
}

impl From<XmlElement> for Sbml {
    fn from(value: XmlElement) -> Self {
        Sbml {
            xml: value.document(),
            sbml_root: value,
        }
    }
}

impl SBase for Sbml {}

/// The `<sbml>` root element is an SBML object like any other, and this is its identity.
///
/// The declaration differs from the one every other type gets from `#[sbml_object(..)]` in exactly
/// one way: **the namespace is a set**, not a single URL. An SBML document root legitimately carries
/// the namespace of its level and version (Level 1 through Level 3, see
/// [`SBML_ROOT_NAMESPACES`]), and this crate recognizes all of them, so
/// a type test that accepted only the Level 3 Version 2 namespace would reject documents the crate
/// reads and validates happily. The set lives in a public data constant, so a level module that
/// teaches the crate a new root namespace extends data rather than this code.
///
/// [`ELEMENT_NAMESPACE`](SbmlObject::ELEMENT_NAMESPACE) is therefore **the namespace this crate
/// writes with** ([`NS_SBML_CORE`]) — the namespace of the documents the crate
/// *creates* — and [`matches_element`](SbmlObject::matches_element) is what decides recognition,
/// accepting any URL of the constant.
///
/// Like [`Sbml::default`], whose element is a fresh, valid skeleton of the Level 3 Version 2
/// namespace, this hand-written identity declares **no constructor**: its `CONSTRUCT` is `None`,
/// because the root is built either by reading a document or by [`Sbml::default`], and a
/// `new_empty(document)` would have to invent a level and a version.
impl SbmlObject for Sbml {
    const ELEMENT_NAME: &'static str = "sbml";
    const ELEMENT_NAMESPACE: Namespace = NS_SBML_CORE;

    fn matches_element(element: &XmlElement) -> bool {
        element.tag_name() == Self::ELEMENT_NAME
            && SBML_ROOT_NAMESPACES.contains(&element.namespace_url().as_str())
    }
}

/// The severity a [rule](crate::RuleDescriptor) assigns to the issues it produces.
///
/// The severity is a property of the rule rather than of the report: an issue inherits it from its
/// [`RuleDescriptor`], so one rule cannot be an error in one place and a note
/// in another.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum SbmlIssueSeverity {
    /// An issue that makes the document impossible to read correctly (e.g. a function is
    /// used but not declared).
    Error,
    /// An issue that suggests a possible error but does not necessarily make the document
    /// invalid (e.g. a variable is declared but never used).
    Warning,
    /// A suggestion that would improve the document but does not represent a significant
    /// issue (e.g. a property is included when it does not have to be, or unknown tags
    /// or attributes are present in the document, e.g. due to the use of unofficial extensions).
    Info,
}

#[cfg(test)]
mod tests {
    use crate::constants::namespaces::{
        NS_EMPTY, NS_FBC, NS_HTML, NS_LAYOUT, NS_SBML_CORE, SBML_ROOT_NAMESPACES, URL_PACKAGE_FBC,
        URL_PACKAGE_LAYOUT, URL_SBML_CORE,
    };
    use crate::constraint::FbcSpecies;
    use crate::core::sbase::SbmlUtils;
    use crate::core::validation::SbmlValidable;
    use crate::core::RuleTypes::Assignment;
    use crate::core::{
        AlgebraicRule, AssignmentRule, BaseUnit, Compartment, Constraint, Delay, Event,
        EventAssignment, FunctionDefinition, InitialAssignment, KineticLaw, LocalParameter, MetaId,
        Model, ModifierSpeciesReference, Parameter, Priority, RateRule, Reaction, Rule, RuleTypes,
        SBase, SId, SboTerm, SimpleSpeciesReference, Species, SpeciesReference, Trigger, Unit,
        UnitDefinition,
    };
    use crate::layout::GeneralGlyph;
    use crate::math::Math;
    use crate::validation::RuleKind;
    use crate::xml::{
        Document, OptionalXmlChild, OptionalXmlProperty, RequiredDynamicChild,
        RequiredDynamicProperty, RequiredXmlChild, RequiredXmlProperty, SbmlObject, XmlChild,
        XmlChildDefault, XmlDefault, XmlElement, XmlProperty, XmlSubtype, XmlSupertype, XmlWrapper,
    };
    use crate::RuleSet;
    use crate::{Sbml, SbmlIssueSeverity};
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// The content of a document of the SBML syntactic test suite, or `None` when the corpus is
    /// not unpacked.
    ///
    /// The corpus is downloaded, not part of the repository (see `VALIDATION.md`), so a test that
    /// reads a document of it has to be able to skip: it prints why and returns, which is the same
    /// convention as `scripts/full_corpus_parity.sh` and `scripts/verify.sh` (a corpus gate reports
    /// a skip and exits successfully). A test that skips proves nothing about the corpus, but it
    /// must not make `cargo test` fail on a fresh checkout.
    fn corpus_document(name: &str) -> Option<String> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("syntactic")
            .join(name);
        match std::fs::read_to_string(&path) {
            Ok(source) => Some(source),
            Err(_) => {
                println!(
                    "skipping: the document {} is not unpacked (see VALIDATION.md)",
                    path.display()
                );
                None
            }
        }
    }

    /// Utility method to create [SId] objects used during testing (we assume the ID is valid).
    pub fn sid(value: &str) -> SId {
        SId::try_from(value).unwrap()
    }

    pub fn mid(value: &str) -> MetaId {
        MetaId::try_from(value).unwrap()
    }

    /// Checks `SbmlDocument`'s properties such as `version` and `level`.
    /// Additionally, checks if `Model` retrieval returns correct child.
    #[test]
    pub fn test_document() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();

        let level = doc.level().get();
        let version = doc.version().get();

        assert_eq!(
            level, 3,
            "Wrong level of SBML.\nActual: {}\nExpected: {}",
            level, 3
        );
        assert_eq!(
            version, 2,
            "Wrong version of SBML.\nActual: {}\nExpected: {}",
            version, 2
        );

        let model = doc.model().get().unwrap();
        let model_id = model.id().get().unwrap();
        assert_eq!(model_id.as_str(), "model_id", "Wrong model.");
    }

    /// Tests read/write operations on `OptionalProperty<>`.
    /// Attempts to remove and create a new custom `OptionalProperty<>`.
    #[test]
    pub fn test_optional_property() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let property = model.id();

        assert!(property.is_set(), "Id is not set but it should be.");
        assert_eq!(
            property.simple_name(),
            "id",
            "Wrong name of the <id> property."
        );
        assert_eq!(
            property.element().xml_element().element(),
            model.xml_element().element(),
            "Wrong underlying element of the <id> property."
        );
        // try reading the <id> property
        let property_val = property.get();
        assert!(
            property_val.is_some(),
            "The <id> property is not set but it should be."
        );
        assert_eq!(
            property_val.unwrap().as_str(),
            "model_id",
            "Wrong value of the <id> property."
        );
        // try clearing the <id> property
        property.clear().unwrap();
        assert!(
            !property.is_set(),
            "The <id> property should be unset (cleared)."
        );
        assert!(property.get().is_none());
        let property_val = property.get();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );
        let property_val = property.get_raw();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );

        // try overwriting the <id> property
        let opt_model_id = sid("optional_model_id");
        property.set_some(&opt_model_id);
        let property_val = property.get();
        assert_eq!(
            property_val,
            Some(opt_model_id),
            "Wrong value of the <id> property."
        );

        let raw_model_id = sid("raw_model_id");
        property.set_raw("raw_model_id".to_string()).unwrap();
        let property_val = property.get();
        assert_eq!(
            property_val,
            Some(raw_model_id),
            "Wrong value of the <id> property."
        );
    }

    /// Tests read/write operations on `RequiredProperty<>`.
    /// Attempts to remove and create a new custom `RequiredProperty<>`.
    #[test]
    pub fn test_required_property() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // create a new required property
        let property: RequiredDynamicProperty<'_, String> =
            RequiredDynamicProperty::new(model.xml_element(), "required_property");
        assert!(
            !property.is_set(),
            "Required property shouldn't be set at this point."
        );
        assert_eq!(
            property.simple_name(),
            "required_property",
            "Wrong name of the required property."
        );
        assert_eq!(
            property.element().xml_element().element(),
            model.xml_element().element(),
            "Wrong underlying element of the required property."
        );
        // try to write and read to/from property
        property.set(&"REQ_12345".to_string());
        let property_val = property.get();
        assert_eq!(
            property_val, "REQ_12345",
            "Wrong value of required property."
        );
        let property_val = property.get_raw();
        assert!(property_val.is_some());
        assert_eq!(
            property_val,
            Some("REQ_12345".to_string()),
            "Wrong value of the required property."
        );
        // try to clear the property
        property.clear().unwrap();
        assert!(
            !property.is_set(),
            "Property shouldn't be set at this point."
        );
        // and write a new value to the property
        property.set_raw("new_req_value".to_string()).unwrap();
        let property_val = property.get();
        assert_eq!(
            property_val, "new_req_value",
            "Wrong value of the required property."
        );
    }

    /// Tests get/set operations on `OptionalChild<>`.
    /// Attempts to remove and create a new custom `OptionalChild<>`.
    #[test]
    pub fn test_optional_child() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // get child
        let notes = model.notes();
        assert!(notes.is_set(), "Notes in Model is not set.");
        assert_eq!(notes.simple_name(), "notes", "Wrong name of Notes child.");
        assert_eq!(
            notes.parent().xml_element().element(),
            model.xml_element().element(),
            "Wrong parent of Notes child."
        );
        // get child value
        let notes_elem = notes.get();
        assert!(notes_elem.is_some(), "Notes does not contain any element.");
        assert_eq!(
            notes_elem.unwrap().tag_name(),
            "notes",
            "Wrong name of Notes child."
        );
        // clear child
        let notes_elem = notes.clear();
        assert!(notes_elem.is_some(), "Old notes child is missing");
        assert!(!notes.is_set(), "Notes are still present after clear.");

        // set child
        let xml_element =
            XmlElement::new_qualified(model.xml_element().document(), "notes", NS_SBML_CORE);
        let notes_elem = notes.set(xml_element);
        assert!(notes_elem.is_none(), "Old Notes should be empty.");
        assert!(notes.is_set(), "Notes should be set.");
    }

    /// Tests get/set operations on `RequiredChild<>`.
    /// Attempts to remove and create a new custom `RequiredChild<>`.
    #[test]
    pub fn test_required_child() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // get child
        let req_child: RequiredDynamicChild<'_, XmlElement> =
            RequiredDynamicChild::new(model.xml_element(), "required");
        assert!(req_child.get_raw().is_none());
        assert_eq!(req_child.simple_name(), "required");
        assert_eq!(
            req_child.parent().xml_element().element(),
            model.xml_element().element()
        );
        let xml_element =
            XmlElement::new_qualified(model.xml_element().document(), "required", NS_EMPTY);
        let inner_element = xml_element.element().clone();
        // set child
        req_child.set_raw(xml_element.clone());
        assert!(req_child.get_raw().is_some());
        xml_element.set_text_content("Some additional content");
        let old_child = req_child.set(xml_element.clone());
        assert_eq!(old_child.element(), &inner_element);
        assert!(req_child.get_raw().is_some());
        assert_eq!(req_child.get().text_content(), "Some additional content");
        req_child.clear_raw();
        assert!(req_child.get_raw().is_none());
    }

    /// Tests get/set operations on special case of children `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`. Checks if addition/removal/get/set methods work correctly
    /// on lists. Attempts to remove and create a new custom `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`.
    #[test]
    pub fn test_lists() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let list = model.compartments();

        assert!(list.is_set());
        assert_eq!(list.simple_name(), "listOfCompartments");
        assert_eq!(
            list.parent().xml_element().element(),
            model.xml_element().element()
        );
        let content = list.get();
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(!content.is_empty());
        assert_eq!(content.len(), 1);
        let compartment1 = content.get(0);
        assert!(compartment1.constant().get());
        assert_eq!(compartment1.id().get().as_str(), "comp1");
        let compartment2: Compartment = Compartment::default(compartment1.xml_element().document());
        compartment2
            .constant()
            .set_raw("false".to_string())
            .unwrap();
        compartment2.id().set_raw("comp2".to_string()).unwrap();
        content.insert(1, compartment2.clone());
        assert_eq!(content.len(), 2);
        assert_eq!(
            content.get(0).xml_element().element(),
            compartment1.xml_element().element()
        );
        assert_eq!(
            content.get(1).xml_element().element(),
            compartment2.xml_element().element()
        );
        content.remove(0);
        assert_eq!(content.len(), 1);
        assert_eq!(
            content.get(0).xml_element().element(),
            compartment2.xml_element().element()
        );
        content.push(compartment1.clone());
        assert_eq!(content.len(), 2);
        assert_eq!(
            content.get(0).xml_element().element(),
            compartment2.xml_element().element()
        );
        assert_eq!(
            content.get(1).xml_element().element(),
            compartment1.xml_element().element()
        );
        content.pop();
        assert_eq!(content.len(), 1);
        assert_eq!(
            content.get(0).xml_element().element(),
            compartment2.xml_element().element()
        );
    }

    #[test]
    pub fn test_build_doc() {
        let sbml_doc = Sbml::default();
        let new_model = Model::default(sbml_doc.xml.clone());

        // set default model element
        sbml_doc.model().set(new_model);
        let model = sbml_doc.model().get().unwrap();
        model.id().set_some(&sid("model_id"));
        model.name().set_some(&"test model No. 1".to_string());
        model
            .sbo_term()
            .set_some(&SboTerm::try_from("SBO:0003204").unwrap());
        model.meta_id().set_some(&mid("MT-TEST-MODEL-NO1"));
        model.notes().set(XmlElement::new_qualified(
            model.xml_element().document(),
            "notes",
            NS_SBML_CORE,
        ));
        model.annotation().set(XmlElement::new_qualified(
            model.xml_element().document(),
            "annotation",
            NS_SBML_CORE,
        ));

        // set default notes for model
        let notes = model.notes().get().unwrap();
        notes.set_text_content("This is a SBML model element.");

        // set default annotation for model
        let annotation = model.annotation().get().unwrap();
        annotation.set_text_content("This is a SBML annotation element.");

        build_function_defs(&model);
        build_unit_defs(&model);
        build_compartments(&model);
        build_species(&model);
        build_parameters(&model);
        build_initial_assignments(&model);
        build_rules(&model);
        build_constraints(&model);
        build_reactions(&model);
        build_events(&model);

        let _ = sbml_doc.write_path("test-inputs/sbml_build_test.sbml");

        // The built document must serialize into XML that parses back into a document with the
        // same validation outcome, and serializing it again must be byte-identical.
        let xml = sbml_doc.to_xml_string().unwrap();
        let reread = Sbml::read_str(xml.as_str()).unwrap();
        assert_eq!(
            sorted_issues(&sbml_doc.validate().unwrap()),
            sorted_issues(&reread.validate().unwrap())
        );
        assert_eq!(xml, reread.to_xml_string().unwrap());

        // Clean up the test file.
        std::fs::remove_file("test-inputs/sbml_build_test.sbml").unwrap();
    }

    /// A helper that turns a validation report into a comparable, order-independent summary.
    fn sorted_issues(issues: &[crate::SbmlIssue]) -> Vec<String> {
        let mut summary = issues
            .iter()
            .map(|it| format!("{}|{:?}|{}", it.rule(), it.severity(), it.message()))
            .collect::<Vec<_>>();
        summary.sort();
        summary
    }

    /// An edit of a document read from disk must survive a serialization round trip: the result
    /// parses back into a document with the same validation outcome, and serializing it a second
    /// time must be byte-identical.
    #[test]
    pub fn test_write_round_trip() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        model.name().set_some(&"round trip model".to_string());
        model.notes().set(XmlElement::new_qualified(
            model.xml_element().document(),
            "notes",
            NS_SBML_CORE,
        ));
        model
            .notes()
            .get()
            .unwrap()
            .set_text_content("A round-trip note.");

        let before = sorted_issues(&doc.validate().unwrap());

        let xml = doc.to_xml_string().unwrap();
        let reread = Sbml::read_str(xml.as_str()).unwrap();
        assert_eq!(before, sorted_issues(&reread.validate().unwrap()));

        // A second serialization of the re-read document is byte-identical to the first one,
        // because the XML layer orders namespace declarations deterministically.
        assert_eq!(xml, reread.to_xml_string().unwrap());
        assert_eq!(model.name().get().unwrap(), "round trip model");
    }

    /// Reading a document through the XML layer honours a `cancel-this` trigger: a trigger that
    /// has already fired aborts the read with `SbmlError::Cancelled`.
    #[test]
    pub fn test_read_cancellation() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model/>
</sbml>"#;

        let trigger = cancel_this::CancelAtomic::new();
        trigger.cancel();
        let result = cancel_this::on_atomic(trigger, || Sbml::read_str(document));
        assert!(matches!(result, Err(crate::SbmlError::Cancelled(_))));

        // Without a fired trigger, the same document reads without an error.
        assert!(Sbml::read_str(document).is_ok());
    }

    /// A cancellation trigger that reports a cancellation only on the `fire_at`-th consultation.
    ///
    /// [`crate::cancellation::CancellationCheck`] consults the triggers on the first step and then
    /// once every `CHECK_INTERVAL` (1024) steps, so this trigger fires only after at least
    /// `1024 * (fire_at - 1)` steps of the operation have been accounted for. An operation that
    /// reports a cancellation therefore proves that it ticks *inside* its walk, not only once when
    /// it starts.
    #[derive(Clone)]
    struct FireOnConsultation {
        consultations: Arc<AtomicUsize>,
        fire_at: usize,
    }

    impl FireOnConsultation {
        fn new(fire_at: usize) -> Self {
            FireOnConsultation {
                consultations: Arc::new(AtomicUsize::new(0)),
                fire_at,
            }
        }

        /// How many times the triggers of this operation have been consulted so far.
        fn consultations(&self) -> usize {
            self.consultations.load(Ordering::SeqCst)
        }
    }

    impl cancel_this::CancellationTrigger for FireOnConsultation {
        fn is_cancelled(&self) -> bool {
            self.consultations.fetch_add(1, Ordering::SeqCst) + 1 >= self.fire_at
        }

        fn type_name(&self) -> &'static str {
            "FireOnConsultation"
        }
    }

    /// A document with `parameters` `<parameter>` elements, in the core namespace.
    fn document_with_parameters(parameters: usize) -> String {
        let mut document = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model id="m"><listOfParameters>"#,
        );
        for index in 0..parameters {
            document.push_str(&format!(r#"<parameter id="p{index}" constant="true"/>"#));
        }
        document.push_str("</listOfParameters></model></sbml>");
        document
    }

    /// A document with a type-check issue *inside* its model is validated, not skipped: `validate`
    /// returns a report instead of panicking, and the report names the element it could not walk.
    ///
    /// The document is a real Level 3 Version 1 corpus document, read as its own level (no namespace
    /// translation), and its type-check problem is a genuine one: a `<curve>` inside a
    /// `<referenceGlyph>`, which the layout package does not permit there. The report carries both
    /// the problem and the `not-validated` note, and - the point of the contract - the issues of the
    /// *rest* of the document, so the guard is per element rather than per document.
    #[test]
    pub fn test_a_type_check_issue_does_not_skip_the_document() {
        let Some(source) = corpus_document("layout-21103/layout-21103-fail-01-01-sev2-l3v1.xml")
        else {
            return;
        };
        let document = Sbml::read_str(&source).expect("the corpus document is well-formed");
        assert!(
            document.model().get().is_some(),
            "a Level 3 Version 1 document resolves its model"
        );
        let issues = document.validate().expect("validation cannot be cancelled");

        let note = issues
            .iter()
            .find(|issue| issue.rule() == "not-validated")
            .unwrap_or_else(|| {
                panic!(
                    "the document must name the subtree it did not validate: {:?}",
                    issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
                )
            });
        assert_eq!(note.kind(), RuleKind::LibraryLint);
        assert_eq!(note.severity(), SbmlIssueSeverity::Info);
        assert!(
            note.message().contains("was not validated"),
            "the note has to say what happened: {}",
            note.message()
        );
        assert!(
            note.message().contains("referenceGlyph"),
            "the note names the element it skipped: {}",
            note.message()
        );
        // The walk of the rest of the document is not suppressed: the document reports the rule of
        // the layout package whose element is broken (`layout-21103`, the two `<curve>` children of
        // the glyph), and that report is on the skipped element's own element, while the issues
        // below are on others.
        assert!(
            issues.iter().any(|issue| issue.rule() == "layout-21103"),
            "the document is still walked outside the skipped subtree: {:?}",
            issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
        );
    }

    /// A problem of the *root* element is reported without skipping anything: the report carries
    /// the root-level rule and no `not-validated` note.
    ///
    /// The two documents are the corpus documents whose root element is not in a published SBML
    /// namespace (one with no namespace at all, one in a foreign namespace), which is the condition
    /// of rule 20101.
    #[test]
    pub fn test_a_type_check_issue_on_the_root_leaves_the_walk_intact() {
        for name in [
            "20101/20101-fail-01-08-sev2-l3v2.xml",
            "20101/20101-fail-01-09-sev2-l3v2.xml",
        ] {
            let Some(source) = corpus_document(name) else {
                return;
            };
            let document = Sbml::read_str(&source).expect("the corpus document is well-formed");
            let issues = document.validate().expect("validation cannot be cancelled");
            assert!(
                issues.iter().any(|issue| issue.rule() == "20101"),
                "{name} declares a root namespace no SBML specification defines: {:?}",
                issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
            );
            assert!(
                !issues.iter().any(|issue| issue.rule() == "not-validated"),
                "a root-level problem does not skip a subtree, so nothing is named as not \
                 validated: {:?}",
                issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
            );
        }
    }

    /// The rule ids of a document's report, in order.
    fn reported_rules(document: &Sbml) -> Vec<String> {
        document
            .validate()
            .expect("validation cannot be cancelled")
            .into_iter()
            .map(|it| it.rule().to_string())
            .collect()
    }

    /// The *specification* rule ids of a document's report, in order (the library lints are not
    /// part of what a document is judged by).
    fn reported_spec_rules(document: &Sbml) -> Vec<String> {
        reported_rules(document)
            .into_iter()
            .filter(|it| RuleSet::CORE.is_spec_rule(it))
            .collect()
    }

    /// The root rules, on the corpus's own documents.
    ///
    /// `20101` is the namespace half of the root rules and `20102`/`20103` are its two attribute
    /// halves, so the suite's documents pin all three: a document that declares a Level 1 level
    /// under a Level 2 namespace reports `20102` and not `20103` (`20102-fail-01-01-sev2-l1v1`),
    /// while one that declares a Level 2 version under the Level 2 Version 1 namespace reports
    /// `20103` and not `20102` (`20103-fail-01-01-sev2-l2v2`), and one whose root element is in no
    /// namespace (or in a foreign one) reports `20101` alone (`20101-fail-01-01`, `-05`, `-06`).
    ///
    /// The two documents of the `99101` family whose expectation file lists only the reference
    /// implementation's own `99101` are the documented differences of these rules: the crate reports
    /// the inconsistent level/version anyway, which the suite suppresses when the declared pair is
    /// not a published one.
    #[test]
    fn the_root_rules_report_what_the_namespace_and_the_attributes_disagree_about() {
        const CASES: &[(&str, &[&str])] = &[
            // 20101: the root element's own namespace is not a published SBML namespace.
            ("20101/20101-fail-01-01-sev2-l2v2.xml", &["20101"]),
            ("20101/20101-fail-01-04-sev2-l3v1.xml", &["20101"]),
            // ... declared only under a prefix, so the root itself is in no namespace.
            ("20101/20101-fail-01-05-sev2-l2v2.xml", &["20101"]),
            // ... the root is in a foreign namespace.
            ("20101/20101-fail-01-06-sev2-l3v1.xml", &["20101"]),
            ("20101/20101-fail-01-09-sev2-l3v2.xml", &["20101"]),
            // ... an unpublished variant of a published namespace.
            ("99101/99101-fail-01-01-20101-sev2-l2v6.xml", &["20101"]),
            // 20102: the declared level against the namespace's level.
            ("20102/20102-fail-01-01-sev2-l1v1.xml", &["20102"]),
            ("20102/20102-fail-01-03-sev2-l2v2.xml", &["20102"]),
            ("20102/20102-fail-01-04-sev2-l2v2.xml", &["20102"]),
            // ... and the missing attribute, which the root's own type check reports.
            ("20102/20102-fail-01-05-sev2-l3v1.xml", &["20102"]),
            ("20102/20102-fail-01-06-sev2-l2v1.xml", &["20102"]),
            // 20103: the declared version against the namespace's version.
            ("20103/20103-fail-01-01-sev2-l2v2.xml", &["20103"]),
            ("20103/20103-fail-01-02-sev2-l2v1.xml", &["20103"]),
            ("20103/20103-fail-01-05-sev2-l2v2.xml", &["20103"]),
            // ... a version Level 1 never published, under the version-less Level 1 namespace.
            ("20103/20103-fail-01-03-99101-sev2-l1v3.xml", &["20103"]),
            ("20103/20103-fail-01-06-99101-sev2-l1v4.xml", &["20103"]),
            ("20103/20103-fail-01-07-99101-sev2-l3v3.xml", &["20103"]),
            ("20103/20103-fail-01-08-sev2-l3v2.xml", &["20103"]),
            // A document that declares a Level 1 level under a Level 2 Version 3 namespace: the
            // level half is what disagrees, so 20102 is what the suite reports too.
            ("99101/99101-fail-01-03-20102-sev2-l1v3.xml", &["20102"]),
        ];

        // The documented differences: the suite suppresses the two attribute halves when the
        // declared pair is not a published combination (it reports its own `99101` instead), while
        // the crate reports the inconsistency it sees.
        const DOCUMENTED_DIFFERENCES: &[(&str, &[&str])] = &[
            ("99101/99101-fail-01-02-sev2-l4v2.xml", &["20102"]),
            ("99101/99101-fail-01-04-sev2-l5v9.xml", &["20102", "20103"]),
        ];

        for (name, expected) in CASES.iter().chain(DOCUMENTED_DIFFERENCES.iter()) {
            let Some(source) = corpus_document(name) else {
                return;
            };
            let document = Sbml::read_str(&source).expect("the corpus document is well-formed");
            assert_eq!(
                reported_spec_rules(&document),
                expected.iter().map(|it| it.to_string()).collect::<Vec<_>>(),
                "{name}: the root rules of the document"
            );
        }
    }

    /// A Level 2 document is read with *its* tables: the attributes Level 2 adds and the ones it
    /// does not have are both decided by the document's own version.
    ///
    /// `20411-pass-00-02-l2v1.xml` is the fixture for the second half of the third case: it carries
    /// `offset` on a `<unit>`, which `sbml-level-2/version-1/schema/sbml.xsd:132` declares (and
    /// which the document's own comment notes is deprecated from Version 2 on), so it must report
    /// nothing - while the same attribute on a Version 2 document would be an unknown attribute.
    #[test]
    fn a_level_2_document_is_read_with_its_own_versions_tables() {
        const CASES: &[(&str, &[&str])] = &[
            // Level 2 Version 1: `offset` is allowed (20411), `charge` is allowed (20411), and the
            // document is a pass document of the suite.
            ("20411/20411-pass-00-02-l2v1.xml", &[]),
            // Level 2 Version 1: `spatialSizeUnits` is allowed on a species.
            ("20421/20421-pass-00-01-l2v1.xml", &[]),
            // Level 2 Version 4: a species without `constant`/`boundaryCondition`/
            // `hasOnlySubstanceUnits` is valid (they are optional with a default), and its report
            // is the rule the document is about.
            ("20409/20409-fail-01-03-sev2-l2v4.xml", &["20409"]),
        ];

        for (name, expected) in CASES {
            let Some(document) = corpus_document(name) else {
                return;
            };
            let document = Sbml::read_str(&document).expect("the corpus document parses");
            let rules = reported_spec_rules(&document);
            for rule in *expected {
                assert!(
                    rules.contains(&rule.to_string()),
                    "{name} must report {rule}: {rules:?}"
                );
            }
            // No *table* problem on any of them: the unknown-attribute and unknown-child ids are
            // what the Level 2 tables exist to keep out of a valid document's report.
            for rule in &rules {
                assert!(
                    !matches!(
                        rule.as_str(),
                        "20421" | "20517" | "20623" | "21132" | "10102" | "21104"
                    ),
                    "{name} must not report a table problem, but reports {rule}"
                );
            }
        }
    }

    /// The eight published SBML document roots, and what this crate does with each of them.
    ///
    /// This is the contract of the object model in one place: a document of a level and version the
    /// crate models resolves its model and is validated against that pair's tables; a document of a
    /// published level the crate does not model yet resolves **no** model and is named by the
    /// `unmodelled-level` lint, so a caller is never told "no issues" about a document that nothing
    /// checked. Enabling a level (the `l1`/`l2` goals) is a change to this table, which is what
    /// makes it visible.
    #[test]
    fn every_published_root_is_read_and_the_unmodelled_ones_are_named() {
        // (namespace, level, version, modelled by this crate)
        const ROOTS: &[(&str, &str, &str, bool)] = &[
            ("http://www.sbml.org/sbml/level1", "1", "1", false),
            ("http://www.sbml.org/sbml/level1", "1", "2", false),
            ("http://www.sbml.org/sbml/level2", "2", "1", true),
            ("http://www.sbml.org/sbml/level2/version2", "2", "2", true),
            ("http://www.sbml.org/sbml/level2/version3", "2", "3", true),
            ("http://www.sbml.org/sbml/level2/version4", "2", "4", true),
            ("http://www.sbml.org/sbml/level2/version5", "2", "5", true),
            (
                "http://www.sbml.org/sbml/level3/version1/core",
                "3",
                "1",
                true,
            ),
            (
                "http://www.sbml.org/sbml/level3/version2/core",
                "3",
                "2",
                true,
            ),
        ];
        for (namespace, level, version, modelled) in ROOTS {
            let source = format!(
                r#"<sbml xmlns="{namespace}" level="{level}" version="{version}"><model/></sbml>"#
            );
            let document = Sbml::read_str(&source).expect("the document is well-formed");
            // The level and version are readable whichever the root is, which is what makes the
            // `unmodelled-level` lint able to name the pair it could not read.
            assert_eq!(document.level().get_raw().as_deref(), Some(*level));
            assert_eq!(document.version().get_raw().as_deref(), Some(*version));
            assert_eq!(
                document.model().get().is_some(),
                *modelled,
                "the object model of {level}v{version} must resolve exactly when the crate models it"
            );

            let rules = reported_rules(&document);
            if *modelled {
                assert!(
                    rules.is_empty(),
                    "{level}v{version} of a bare model reports nothing: {rules:?}"
                );
            } else {
                assert_eq!(
                    rules,
                    vec!["unmodelled-level".to_string()],
                    "a document of a published but unmodelled level is named, and nothing else is \
                     checked"
                );
            }
        }
    }

    /// A root whose namespace no specification defines resolves no model and reports rule 20101,
    /// and *only* rule 20101: nothing in such a document can be interpreted, so nothing else is
    /// checked.
    ///
    /// This is the restated form of the earlier "a foreign root still parses and reports" note: the
    /// guarantee is now executable, and it is what the corpus's `20101-fail-*` documents expect
    /// (`20101-fail-01-05` binds the SBML namespace under a prefix that the `<sbml>` element itself
    /// does not use, `20101-fail-01-06` puts the root in a foreign namespace).
    #[test]
    fn a_root_outside_the_published_namespaces_is_reported_as_20101() {
        for (label, source) in [
            (
                "a foreign namespace",
                r#"<sbml xmlns="http://example.org/not-sbml" level="3" version="2"><model/></sbml>"#,
            ),
            (
                "no namespace at all",
                r#"<sbml level="3" version="2"><model/></sbml>"#,
            ),
            (
                "the SBML namespace under an unused prefix",
                r#"<sbml xmlns:sbml="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2"><model/></sbml>"#,
            ),
            (
                "an unpublished variant of a published namespace",
                r#"<sbml xmlns="http://www.sbml.org/sbml/level2/version6" level="2" version="6"><model/></sbml>"#,
            ),
        ] {
            let document = Sbml::read_str(source).expect("the document is well-formed");
            assert!(
                document.model().get().is_none(),
                "{label}: no object model resolves"
            );
            let rules = reported_rules(&document);
            assert_eq!(
                rules.first().map(String::as_str),
                Some("20101"),
                "{label}: the root rule comes first: {rules:?}"
            );
            assert_eq!(
                rules
                    .iter()
                    .filter(|it| crate::RuleSet::CORE.is_spec_rule(it))
                    .collect::<Vec<_>>(),
                vec!["20101"],
                "{label}: no *content* rule is reported, because nothing in such a document can be \
                 interpreted ({rules:?})"
            );
        }
    }

    /// When the declared `level`/`version` attributes and the root namespace disagree, the
    /// **attributes** decide which tables the document is read with, and the root rules report the
    /// disagreement (rules 20102 and 20103 are the consistency halves, which the crate reports for a
    /// missing attribute today).
    ///
    /// The corpus is what pins the direction: `20102-fail-01-01` declares the Level 1 namespace and
    /// `level="1" version="1"`, has Level 1 content (`<specie>`, `<specieReference>`), and its
    /// expectation file lists *only* rule 20102 - so the reference implementation reads the content
    /// as the level the attribute names, not the one the namespace does.
    #[test]
    fn the_declared_level_and_version_decide_the_tables() {
        // The Version 2 namespace with `version="1"`: read as Version 1, where `fast` is a legal
        // reaction attribute (rule 21110 of Version 1 requires it, in fact).
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3"
            version="1"><model><listOfReactions><reaction id="r" reversible="true"
            fast="false"/></listOfReactions></model></sbml>"#;
        let document = Sbml::read_str(source).expect("the document is well-formed");
        assert!(document.model().get().is_some());
        assert!(
            !reported_rules(&document).contains(&"21110".to_string()),
            "`fast` is a Version 1 attribute, so a document declaring `version=\"1\"` may use it"
        );

        // The Version 1 namespace with `version="2"`: read as Version 2, which removed `fast`.
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3"
            version="2"><model><listOfReactions><reaction id="r" reversible="true"
            fast="false"/></listOfReactions></model></sbml>"#;
        let document = Sbml::read_str(source).expect("the document is well-formed");
        assert!(document.model().get().is_some());
        assert!(
            reported_rules(&document).contains(&"21110".to_string()),
            "`fast` was removed in Version 2, so a document declaring `version=\"2\"` may not use it"
        );
    }

    /// The other direction of the Version 1/Version 2 delta, on the element the corpus exercises
    /// it with: Version 2 added the optional `id` and `name` attributes to a `unit`, so a Version 1
    /// document that has one is invalid (rule 20421), while the same document in Version 2 is not
    /// (`syntactic/20421/20421-fail-01-05-sev2-l3v1.xml` is the corpus document).
    #[test]
    fn the_attributes_version_2_added_are_reported_in_version_1() {
        let body = r#"<model><listOfUnitDefinitions><unitDefinition id="u"><listOfUnits><unit id="x"
            kind="mole" exponent="1" scale="0" multiplier="1"/></listOfUnits></unitDefinition>
            </listOfUnitDefinitions></model>"#;
        let v1 = format!(
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3" version="1">{body}</sbml>"#
        );
        let v2 = format!(
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">{body}</sbml>"#
        );

        let document = Sbml::read_str(&v1).expect("the document is well-formed");
        assert!(
            reported_rules(&document).contains(&"20421".to_string()),
            "`unit@id` is not permitted in Version 1"
        );
        let document = Sbml::read_str(&v2).expect("the document is well-formed");
        assert!(
            reported_rules(&document).is_empty(),
            "`unit@id` is permitted in Version 2"
        );
    }

    /// Creating a child of a resolved document creates it in *that document's* core namespace, so
    /// editing an older document does not produce a document that mixes two core namespaces.
    #[test]
    fn a_new_child_of_a_version_1_document_joins_its_namespace() {
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3"
            version="1"><model/></sbml>"#;
        let document = Sbml::read_str(source).expect("the document is well-formed");
        let model = document.model().get().expect("the model resolves");
        let compartment = Compartment::new_empty(document.xml_element().document());
        assert_eq!(
            compartment.xml_element().namespace_url(),
            "http://www.sbml.org/sbml/level3/version1/core",
            "a new compartment joins the document's level"
        );
        // The constructor produces an empty element; a Version 1 compartment needs an `id` and
        // `constant` (rule 20517), which the typed properties write.
        compartment
            .id()
            .set(&SId::try_from("c".to_string()).expect("a valid identifier"));
        compartment.constant().set(&true);
        model.compartments().ensure();
        model
            .compartments()
            .get()
            .expect("the list was created")
            .push(compartment);

        // The document is still consistent: the compartment is found again through the typed view,
        // in the namespace of the document, and the document reports no issue (a Version 1 model
        // with one compartment is valid).
        let compartments = model.compartments().get().expect("the list was created");
        assert_eq!(compartments.len(), 1);
        let stored = compartments.get(0);
        assert_eq!(
            stored.xml_element().namespace_url(),
            "http://www.sbml.org/sbml/level3/version1/core"
        );
        assert_eq!(stored.id().get_raw().as_deref(), Some("c"));
        assert!(reported_rules(&document).is_empty());
    }

    /// The headline of the unit, as a test: a real Level 3 Version 1 corpus document resolves its
    /// model and its report is the rule its test case is about, with no `21110` for the `fast`
    /// attribute it carries and no `not-validated` cascade.
    ///
    /// Before this unit the same document resolved **no** model and reported nothing at all, and the
    /// same document with its core namespace rewritten to Version 2 reported `21110` (an unknown
    /// attribute) plus `not-validated`. The `fbc-21207` rule is the one the document's own test case
    /// is about, and it is what remains.
    #[test]
    fn a_version_1_corpus_document_resolves_and_reports_its_own_rule() {
        let Some(source) = corpus_document("fbc-21207/fbc-21207-fail-01-01-sev2-l3v1.xml") else {
            return;
        };
        let document = Sbml::read_str(&source).expect("the corpus document is well-formed");
        assert!(
            document.model().get().is_some(),
            "a Level 3 Version 1 document resolves its model"
        );
        assert_eq!(
            reported_rules(&document),
            vec!["fbc-21207".to_string()],
            "the report is the rule of the document's own test case"
        );
    }

    /// One end-to-end document per modelled level: read a real corpus document of that level,
    /// resolve its model, validate it, and require that a document which is valid *in its own
    /// level* reports no unknown-attribute and no unknown-child issue.
    ///
    /// The Level 3 Version 1 document is `21110-pass-00-01-sev1-l3v1.xml`, which carries
    /// `fast="false"` on its reaction and is the pass case of the rule about that attribute; its
    /// expectation file lists only a libSBML diagnostic, so the library reports nothing at all.
    #[test]
    fn an_end_to_end_document_per_modelled_level() {
        for (level, name) in [
            ("3v1", "21110/21110-pass-00-01-sev1-l3v1.xml"),
            ("3v2", "21110/21110-pass-00-02-sev1-l3v2.xml"),
        ] {
            let Some(source) = corpus_document(name) else {
                return;
            };
            let document = Sbml::read_str(&source).expect("the corpus document is well-formed");
            assert!(
                document.model().get().is_some(),
                "{level}: the model of {name} resolves"
            );
            let issues = document.validate().expect("validation cannot be cancelled");
            assert!(
                issues.is_empty(),
                "{level}: {name} is valid in its own level, so nothing is reported: {:?}",
                issues
                    .iter()
                    .map(|it| (it.rule(), it.message().to_string()))
                    .collect::<Vec<_>>()
            );
        }
    }

    /// The documented detection works, and the guard is per element rather than per document: a
    /// broken element is named and skipped, while the walk of its siblings still runs.
    ///
    /// The document has a compartment without an `id` (a type-check issue: `id` is required), a
    /// second compartment that is fine, and two compartments that share an id - the last pair is a
    /// rule of the *walk*, so its report is the proof that the walk ran.
    #[test]
    pub fn test_the_walk_reports_what_it_skipped_and_keeps_going() {
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
            <model>
                <listOfCompartments>
                    <compartment constant="true"/>
                    <compartment id="good" constant="true"/>
                    <compartment id="twin" constant="true"/>
                    <compartment id="twin" constant="true"/>
                </listOfCompartments>
            </model>
        </sbml>"#;
        let document = Sbml::read_str(source).expect("The document is well formed XML.");
        let issues = document
            .validate()
            .expect("Validation cannot be cancelled here.");
        let rules = issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>();

        // The type check reports the missing `id` of the first compartment...
        assert!(rules.contains(&"20517"), "{rules:?}");
        // ... the walk names the element it did not validate ...
        let note = issues
            .iter()
            .find(|issue| issue.rule() == "not-validated")
            .unwrap_or_else(|| panic!("the skipped element is named: {rules:?}"));
        assert_eq!(note.kind(), RuleKind::LibraryLint);
        assert_eq!(note.severity(), SbmlIssueSeverity::Info);
        assert!(
            note.message().contains("<compartment>"),
            "the note has to name the element: {}",
            note.message()
        );
        // ... and the walk of the rest of the model still runs, which the duplicate `id` proves.
        assert!(rules.contains(&"10301"), "{rules:?}");

        // The other half of the contract: the typed view of the broken element is unusable. The
        // test asserts that documented behavior instead of relying on it silently.
        let model = document.model().get().expect("The document has a model.");
        let broken = model
            .compartments()
            .get()
            .expect("The model has compartments")
            .get(0);
        let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = broken.id().get();
        }));
        assert!(
            read.is_err(),
            "the typed read of a type-inconsistent element has to be unusable"
        );
        // The sibling that is type-consistent is readable: the guard is per element.
        let good = model
            .compartments()
            .get()
            .expect("The model has compartments")
            .get(1);
        assert_eq!(good.id().get_checked(), Ok(Some(sid("good"))));
    }

    /// The other direction: a document with no type-check issue reports no `not-validated` note, so
    /// the note cannot be mistaken for a normal report.
    #[test]
    pub fn test_a_clean_document_reports_no_not_validated_note() {
        let document = Sbml::read_path("test-inputs/model.sbml").expect("readable");
        let issues = document.validate().expect("not cancelled");
        assert!(
            !issues.iter().any(|issue| issue.rule() == "not-validated"),
            "{:?}",
            issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
        );
    }

    /// Validation honours the same `cancel-this` triggers as reading: a trigger that has already
    /// fired aborts the whole call with `SbmlError::Cancelled` instead of producing a report.
    #[test]
    pub fn test_validate_cancellation() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model/>
</sbml>"#;

        // The document is read first, so that the trigger below can only be seen by `validate`.
        let doc = Sbml::read_str(document).expect("The test document is well formed.");

        let trigger = cancel_this::CancelAtomic::new();
        trigger.cancel();
        let result = cancel_this::on_atomic(trigger, || doc.validate());
        assert!(matches!(result, Err(crate::SbmlError::Cancelled(_))));

        // Without a fired trigger, the same document validates without an error.
        assert!(doc.validate().is_ok());
    }

    /// The tick of [`Sbml::validate`] is inside its walks, not only at the start: a trigger that
    /// fires on a *later* consultation aborts the call after the walk has already accounted for
    /// more than a thousand steps.
    ///
    /// The document has 800 `<parameter>` elements, so the call accounts for about 2400 steps —
    /// one per element in the type check, one per element in the `unused-namespace` lint and one
    /// per object in the rule-by-rule walk — and therefore consults the triggers three times: at
    /// the first step of the call and then about every 1024 steps. A trigger that fires on the
    /// second consultation can therefore only be seen by a tick *inside* a walk.
    ///
    /// Only the error is asserted, not the step at which it was produced: the countdown is shared
    /// by all phases, so the exact step (and therefore the phase that supplies it) is an
    /// implementation detail, while "the walk notices a request" is the contract. The document is
    /// large enough that the second consultation is reached with more than 1300 steps to spare, so
    /// the test does not depend on the exact number of steps a phase accounts for; should the step
    /// count ever shrink below the interval, the test fails loudly on the assertion below rather
    /// than passing vacuously.
    #[test]
    pub fn test_validate_cancellation_during_the_walk() {
        let document = document_with_parameters(800);
        let doc = Sbml::read_str(document.as_str()).expect("The test document is well formed.");

        // The document is clean, so the call runs every phase (a report would make it return
        // after the type check and the lints).
        assert!(doc
            .validate()
            .expect("Validation cannot be cancelled here.")
            .is_empty());

        let trigger = FireOnConsultation::new(2);
        let observer = trigger.clone();
        let result = cancel_this::on_trigger(trigger, || doc.validate());
        assert!(
            matches!(result, Err(crate::SbmlError::Cancelled(_))),
            "The second consultation of the walk must abort the validation."
        );
        assert!(
            observer.consultations() >= 2,
            "The walk must consult the triggers more than once, but they were consulted {} time(s).",
            observer.consultations()
        );
    }

    /// The text of every text node of a document, in document order.
    fn document_text_nodes(doc: &Sbml) -> Vec<String> {
        doc.xml_element()
            .document()
            .nodes()
            .into_iter()
            .filter_map(|it| it.text().map(|text| text.as_str().to_string()))
            .collect()
    }

    /// Reading a document keeps its text: the padding inside a `notes`, `annotation` or MathML
    /// payload is still there after reading, and after serializing and reading the document
    /// again.
    #[test]
    pub fn test_read_preserves_text() {
        let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model id="m">
    <notes>
      A   note
        with    padding.
    </notes>
    <annotation>
      <p>  annotation   text  </p>
    </annotation>
    <listOfParameters>
      <parameter id="p" constant="true"/>
    </listOfParameters>
    <listOfInitialAssignments>
      <initialAssignment symbol="p">
        <math xmlns="http://www.w3.org/1998/Math/MathML">
          <apply>
            <plus/>
            <ci> p </ci>
            <cn> 1 </cn>
          </apply>
        </math>
      </initialAssignment>
    </listOfInitialAssignments>
  </model>
</sbml>"#;

        let doc = Sbml::read_str(source).unwrap();
        let model = doc.model().get().unwrap();
        assert_eq!(
            model.notes().get().unwrap().text_content(),
            "\n      A   note\n        with    padding.\n    "
        );
        assert!(model
            .annotation()
            .get()
            .unwrap()
            .text_content()
            .contains("  annotation   text  "));
        let math = model
            .initial_assignments()
            .get()
            .unwrap()
            .get(0)
            .math()
            .get()
            .unwrap();
        let ci = math
            .xml_element()
            .recursive_child_elements_filtered(|it| it.tag_name() == "ci")
            .pop()
            .unwrap();
        assert_eq!(ci.text_content(), " p ");

        let written = doc.to_xml_string().unwrap();
        let reread = Sbml::read_str(written.as_str()).unwrap();
        assert_eq!(document_text_nodes(&doc), document_text_nodes(&reread));
        assert_eq!(written, reread.to_xml_string().unwrap());
    }

    /// A value that SBML *interprets* is read without its surrounding whitespace: identifiers,
    /// numbers, booleans and enumerated values may all be padded in the document.
    #[test]
    pub fn test_padded_values_are_interpreted() {
        let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level=" 3 " version="	2	">
  <model id="m">
    <listOfUnitDefinitions>
      <unitDefinition id="u">
        <listOfUnits>
          <unit kind=" litre " exponent=" 1 " scale=" 0 " multiplier=" 1 "/>
        </listOfUnits>
      </unitDefinition>
    </listOfUnitDefinitions>
    <listOfCompartments>
      <compartment id=" c " constant=" true " size=" 2.5 "/>
    </listOfCompartments>
    <listOfParameters>
      <parameter id="p" constant="true"/>
    </listOfParameters>
    <listOfInitialAssignments>
      <initialAssignment symbol="p">
        <math xmlns="http://www.w3.org/1998/Math/MathML">
          <apply>
            <plus/>
            <cn>1</cn>
            <ci>missing</ci>
          </apply>
        </math>
      </initialAssignment>
    </listOfInitialAssignments>
  </model>
</sbml>"#;

        let doc = Sbml::read_str(source).unwrap();
        assert_eq!(doc.level().get(), 3);
        assert_eq!(doc.version().get(), 2);
        let model = doc.model().get().unwrap();
        assert_eq!(model.id().get().unwrap().as_str(), "m");
        let unit = model
            .unit_definitions()
            .get()
            .unwrap()
            .get(0)
            .units()
            .get()
            .unwrap()
            .get(0);
        assert_eq!(unit.kind().get(), BaseUnit::Litre);
        assert_eq!(unit.exponent().get(), 1.0);
        let compartment = model.compartments().get().unwrap().get(0);
        assert_eq!(compartment.id().get().as_str(), "c");
        assert!(compartment.constant().get());
        assert_eq!(compartment.size().get(), Some(2.5));
    }

    /// A text payload that SBML interprets as an identifier is read without its surrounding
    /// whitespace: the padded and the unpadded document produce the same report.
    ///
    /// The check is discriminating because the identifier used here is *not* part of the model:
    /// a `<ci>` element whose text cannot be parsed as an `SId` is skipped silently by the
    /// formula layer, so a version that did not remove the padding would report one issue fewer.
    #[test]
    pub fn test_padded_math_identifier_is_interpreted() {
        let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model id="m">
    <listOfParameters>
      <parameter id="p" constant="true"/>
    </listOfParameters>
    <listOfInitialAssignments>
      <initialAssignment symbol="p">
        <math xmlns="http://www.w3.org/1998/Math/MathML">
          <apply>
            <plus/>
            <cn>1</cn>
            <ci>missing</ci>
          </apply>
        </math>
      </initialAssignment>
    </listOfInitialAssignments>
  </model>
</sbml>"#;

        let padded = source.replace("<ci>missing</ci>", "<ci>\n  missing\n</ci>");
        assert_ne!(source, padded);

        let plain_report = sorted_issues(&Sbml::read_str(source).unwrap().validate().unwrap());
        assert!(
            plain_report.iter().any(|it| it.starts_with("10215|")),
            "the test is only meaningful if an unknown identifier is reported: {plain_report:?}"
        );
        assert_eq!(
            plain_report,
            sorted_issues(&Sbml::read_str(&padded).unwrap().validate().unwrap())
        );
    }

    fn build_function_defs(model: &Model) {
        let function_defs = model.function_definitions();
        function_defs.ensure();

        let function_defs_list = function_defs.get().unwrap();
        function_defs_list.id().set_some(&sid("FunDefsListID"));
        function_defs_list
            .name()
            .set_some(&"FunDefsListNAME".to_string());
        function_defs_list.push(FunctionDefinition::default(model.xml_element().document()));
        function_defs_list.push(FunctionDefinition::default(model.xml_element().document()));
        function_defs_list.push(FunctionDefinition::default(model.xml_element().document()));

        function_defs_list
            .get(0)
            .id()
            .set_some(&sid("functionDef1"));
        function_defs_list
            .get(1)
            .id()
            .set_some(&sid("functionDef2"));
        let fd_top = function_defs_list.top();
        fd_top.id().set_some(&sid("functionDef3"));
        fd_top
            .math()
            .set(Math::default(model.xml_element().document()));
    }

    fn build_unit_defs(model: &Model) {
        let unit_defs = model.unit_definitions();
        unit_defs.ensure();

        let unit_defs_list = unit_defs.get().unwrap();
        unit_defs_list.id().set_some(&sid("UnitDefsListID"));
        unit_defs_list
            .name()
            .set_some(&"UnitDefsListNAME".to_string());
        unit_defs_list.push(UnitDefinition::default(model.xml_element().document()));
        unit_defs_list.push(UnitDefinition::default(model.xml_element().document()));
        unit_defs_list.push(UnitDefinition::default(model.xml_element().document()));

        unit_defs_list.get(0).id().set_some(&sid("unitDef1"));
        unit_defs_list.get(1).id().set_some(&sid("unitDef2"));
        let ud_top = unit_defs_list.top();
        ud_top.id().set_some(&sid("unitDef3_Length"));
        ud_top.name().set_some(&"unitDef3_Length".to_string());

        // set default list of units for unit definition
        ud_top.units().ensure();
        let units_list = ud_top.units().get().unwrap();
        units_list.push(Unit::default(model.xml_element().document()));

        let unit = units_list.top();
        unit.kind().set(&BaseUnit::Metre);
    }

    fn build_compartments(model: &Model) {
        let compartments = model.compartments();
        compartments.ensure();

        let compartments = compartments.get().unwrap();
        compartments.id().set_some(&sid("CompsListID"));
        compartments.name().set_some(&"CompsListNAME".to_string());
        compartments.push(Compartment::default(model.xml_element().document()));
        compartments.push(Compartment::default(model.xml_element().document()));
        compartments.push(Compartment::default(model.xml_element().document()));

        compartments.get(0).id().set(&sid("compartment1"));
        compartments.get(0).constant().set(&false);
        compartments.get(1).id().set(&sid("compartment2"));
        compartments.get(1).constant().set(&false);

        let comp_top = compartments.top();
        comp_top.id().set(&sid("compartment3"));
        comp_top.spatial_dimensions().set_some(&3.0);
        comp_top.size().set_some(&1.0);
        comp_top.units().set_some(&sid("volume"));
        comp_top.constant().set(&true);
    }

    fn build_species(model: &Model) {
        let species = model.species();
        species.ensure();

        let species = species.get().unwrap();
        species.id().set_some(&sid("Species"));
        species.name().set_some(&"SpeciesListNAME".to_string());
        species.push(Species::new(
            model.xml_element().document(),
            &sid("species1"),
            &sid("compartment1"),
        ));
        species.push(Species::new(
            model.xml_element().document(),
            &sid("species2"),
            &sid("compartment2"),
        ));
        species.push(Species::new(
            model.xml_element().document(),
            &sid("species3"),
            &sid("compartment3"),
        ));

        let species_top = species.top();
        species_top.initial_amount().set_some(&10.0);
        species_top.initial_concentration().set_some(&0.5);
        species_top
            .substance_units()
            .set_some(&BaseUnit::Sievert.into());
        species_top.has_only_substance_units().set(&false);
        species_top.boundary_condition().set(&true);
        species_top.constant().set(&false);
        species_top.conversion_factor().set_some(&sid("linear"));
    }

    fn build_parameters(model: &Model) {
        let parameters = model.parameters();
        parameters.ensure();

        let parameters = parameters.get().unwrap();
        parameters.id().set_some(&sid("ParamsListID"));
        parameters.name().set_some(&"ParamsListNAME".to_string());
        parameters.push(Parameter::new(
            model.xml_element().document(),
            &sid("param1"),
            true,
        ));
        parameters.push(Parameter::new(
            model.xml_element().document(),
            &sid("param2"),
            true,
        ));

        let param_top = parameters.top();
        param_top.value().set_some(&15.0);
        param_top.units().set_some(&BaseUnit::Ampere.into());
    }

    fn build_initial_assignments(model: &Model) {
        let assignments = model.initial_assignments();
        assignments.ensure();

        let assignments = assignments.get().unwrap();
        assignments.id().set_some(&sid("InitialAssignmentsListID"));
        assignments
            .name()
            .set_some(&"InitialAssignmentsListNAME".to_string());
        assignments.push(InitialAssignment::new(
            model.xml_element().document(),
            &sid("x"),
        ));
        assignments.push(InitialAssignment::new(
            model.xml_element().document(),
            &sid("x"),
        ));

        assignments.get(0).math().ensure();
        assignments.get(1).math().ensure();
    }

    fn build_rules(model: &Model) {
        let rules = model.rules();
        rules.ensure();

        let rules = rules.get().unwrap();
        rules.id().set_some(&sid("RulesListID"));
        rules.name().set_some(&"RulesListNAME".to_string());
        rules.push(AlgebraicRule::default(model.xml_element().document()).upcast());
        rules.push(AssignmentRule::new(model.xml_element().document(), &sid("z")).upcast());
        rules.push(RateRule::new(model.xml_element().document(), &sid("r")).upcast());

        let algebraic: AlgebraicRule = rules.get(0).downcast();
        algebraic.id().set_some(&sid("rule1"));
        algebraic.name().set_some(&"algebraic".to_string());

        let assignment: AssignmentRule = rules.get(1).downcast();
        assignment.id().set_some(&sid("rule2"));
        assignment.name().set_some(&"assignment".to_string());

        let rate: RateRule = rules.get(2).downcast();
        rate.id().set_some(&sid("rule3"));
        rate.name().set_some(&"rate".to_string());
    }

    fn build_constraints(model: &Model) {
        let constraints = model.constraints();
        constraints.ensure();

        let constraints = constraints.get().unwrap();
        constraints.id().set_some(&sid("ConstraintsListID"));
        constraints
            .name()
            .set_some(&"ConstraintsListNAME".to_string());
        constraints.push(Constraint::default(model.xml_element().document()));
        constraints.push(Constraint::default(model.xml_element().document()));

        constraints.get(0).id().set_some(&sid("constraint1"));
        constraints.get(1).id().set_some(&sid("constraint2"));

        let constraint_top = constraints.top();
        constraint_top.message().set(XmlElement::new_qualified(
            model.xml_element().document(),
            "message",
            NS_HTML,
        ));
        constraint_top.math().ensure();
    }

    fn build_reactions(model: &Model) {
        let reactions = model.reactions();
        reactions.ensure();

        let reactions = reactions.get().unwrap();
        reactions.id().set_some(&sid("ReactionsListID"));
        reactions.name().set_some(&"ReactionsListNAME".to_string());
        reactions.push(Reaction::new(
            model.xml_element().document(),
            &sid("reaction1"),
            true,
        ));

        let reaction = reactions.top();
        reaction.compartment().set_some(&sid("compartment1"));

        let reactants = reaction.reactants();
        reactants.ensure();
        let reactants = reactants.get().unwrap();
        reactants.id().set_some(&sid("ReactantsListID"));
        reactants.push(SpeciesReference::new(
            model.xml_element().document(),
            &sid("species1"),
            true,
        ));
        let reactant = reactants.top();
        reactant.stoichiometry().set_some(&2.0);

        let products = reaction.products();
        products.ensure();
        let products = products.get().unwrap();
        products.id().set_some(&sid("ProductsListID"));
        products.push(SpeciesReference::new(
            model.xml_element().document(),
            &sid("species1"),
            true,
        ));
        let product = products.top();
        product.stoichiometry().set_some(&1.0);

        let modifiers = reaction.modifiers();
        modifiers.ensure();
        let modifiers = modifiers.get().unwrap();
        modifiers.id().set_some(&sid("ModifiersListID"));
        modifiers.push(ModifierSpeciesReference::new(
            model.xml_element().document(),
            &sid("species2"),
        ));

        let kinetic_law = reaction.kinetic_law();
        kinetic_law.set(KineticLaw::default(model.xml_element().document()));
        kinetic_law.get().unwrap().math().ensure();
        let kinetic_law = kinetic_law.get().unwrap();
        let local_params = kinetic_law.local_parameters();
        local_params.ensure();
        let local_params = local_params.get().unwrap();
        local_params.push(LocalParameter::new(
            model.xml_element().document(),
            &sid("localParamID"),
        ));
        let param = local_params.top();
        param.value().set_some(&42.0);
        param.units().set_some(&sid("meter"));
    }

    fn build_events(model: &Model) {
        let events = model.events();
        events.ensure();

        let events = events.get().unwrap();
        events.id().set_some(&sid("EventsListID"));
        events.push(Event::default(model.xml_element().document()));
        events.push(Event::default(model.xml_element().document()));

        events.get(0).use_values_from_trigger_time().set(&true);
        events.get(1).use_values_from_trigger_time().set(&false);

        let event = events.top();

        event
            .trigger()
            .set(Trigger::default(model.xml_element().document()));
        let trigger = event.trigger().get().unwrap();
        trigger.initial_value().set(&true);
        trigger.persistent().set(&true);
        trigger.math().ensure();

        event
            .priority()
            .set(Priority::default(model.xml_element().document()));
        let priority = event.priority().get().unwrap();
        priority.math().ensure();

        event
            .delay()
            .set(Delay::default(model.xml_element().document()));
        let delay = event.delay().get().unwrap();
        delay.math().ensure();

        let event_assignments = event.event_assignments();
        event_assignments.ensure();
        let event_assignments = event_assignments.get().unwrap();
        event_assignments
            .id()
            .set_some(&sid("EventAssignmentsListID"));
        event_assignments.push(EventAssignment::new(
            model.xml_element().document(),
            &sid("evt"),
        ));
        let assignment = event_assignments.top();
        assignment.math().ensure();
    }

    #[test]
    pub fn test_sbase() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model: Model = doc.model().get().unwrap();

        let id = model.id();
        assert!(id.is_set());
        assert_eq!(id.simple_name(), "id");
        assert_eq!(
            id.element().xml_element().element(),
            model.xml_element().element()
        );
        assert_eq!(id.get().unwrap().as_str(), "model_id");
        id.set_some(&sid("new_model_id"));
        assert_eq!(id.get().unwrap().as_str(), "new_model_id");
        id.clear().unwrap();
        assert!(!id.is_set());

        let name = model.name();
        assert!(!name.is_set());
        assert_eq!(name.simple_name(), "name");
        assert_eq!(
            name.element().xml_element().element(),
            model.xml_element().element()
        );
        name.set_some(&"model_name".to_string());
        assert_eq!(name.get().unwrap(), "model_name");
        name.clear().unwrap();
        assert!(!name.is_set());

        let meta_id = model.meta_id();
        assert!(meta_id.is_set());
        assert_eq!(meta_id.simple_name(), "metaid");
        assert_eq!(
            meta_id.element().xml_element().element(),
            model.xml_element().element()
        );
        assert_eq!(
            meta_id.get().unwrap().as_str(),
            "_174907b7-8e1c-47f3-9a50-bb8e4c6ebd0d"
        );
        meta_id.set_some(&mid("new_meta_id_12345"));
        assert_eq!(meta_id.get().unwrap().as_str(), "new_meta_id_12345");
        meta_id.clear().unwrap();
        assert!(!meta_id.is_set());

        let sbo_term = model.sbo_term();
        assert!(!sbo_term.is_set());
        assert_eq!(sbo_term.simple_name(), "sboTerm");
        assert_eq!(
            name.element().xml_element().element(),
            model.xml_element().element()
        );
        sbo_term.set_some(&SboTerm::try_from("SBO:0000014").unwrap());
        assert_eq!(sbo_term.get().unwrap().as_str(), "SBO:0000014");
        sbo_term.clear().unwrap();
        assert!(!sbo_term.is_set());

        let notes = model.notes();
        assert!(notes.is_set());
        assert_eq!(
            notes.parent().xml_element().element(),
            model.xml_element().element()
        );
        assert_eq!(notes.simple_name(), "notes");
        assert!(notes.get().is_some());

        let annotation = model.annotation();
        assert!(annotation.is_set());
        assert_eq!(
            annotation.parent().xml_element().element(),
            model.xml_element().element()
        );
        assert_eq!(annotation.simple_name(), "annotation");
        assert!(annotation.get().is_some());
    }

    #[test]
    pub fn test_function_definitions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let f_defs = model.function_definitions();
        assert!(f_defs.is_set());

        let f_defs = f_defs.get().unwrap();
        assert!(!f_defs.is_empty());
        assert_eq!(f_defs.len(), 46);

        let f_definition = f_defs.get(0);
        assert!(f_definition.annotation().is_set());
        assert!(f_definition.math().is_set());
    }

    #[test]
    pub fn test_unit_definitions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let unit_defs = model.unit_definitions();
        assert!(unit_defs.is_set());

        let unit_defs = unit_defs.get().unwrap();
        assert!(!unit_defs.is_empty());
        assert_eq!(unit_defs.len(), 5);

        let unit_def = unit_defs.get(0);
        let units = unit_def.units();
        assert!(units.is_set());

        let units = units.get().unwrap();
        assert_eq!(units.len(), 1);

        let unit = units.get(0);
        assert_eq!(unit.exponent().get(), 1.0);
        assert_eq!(unit.kind().get(), BaseUnit::Metre);
        assert_eq!(unit.kind().get().to_string(), BaseUnit::Metre.to_string());
        assert_eq!(unit.multiplier().get(), 1.0);
        assert_eq!(unit.scale().get(), 0);
    }

    #[test]
    pub fn test_compartments() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let compartments = model.compartments();
        assert!(compartments.is_set());

        let compartments = compartments.get().unwrap();
        assert!(!compartments.is_empty());
        assert_eq!(compartments.len(), 7);

        let compartment = compartments.get(0);
        assert_eq!(compartment.id().get().as_str(), "Intake");
        assert!(!compartment.units().is_set());
        assert!(compartment.constant().get());
        assert!(compartment.size().is_set());
        assert_eq!(compartment.size().get().unwrap(), 1.0);
        assert!(compartment.spatial_dimensions().is_set());
        assert_eq!(compartment.spatial_dimensions().get().unwrap(), 3.0);
        assert!(!compartment.units().is_set());
    }

    #[test]
    pub fn test_species() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let species = model.species();
        assert!(species.is_set());

        let species = species.get().unwrap();
        assert!(!species.is_empty());
        assert_eq!(species.len(), 51);

        let specie = species.get(0);
        assert_eq!(specie.id().get().as_str(), "species_1");
        assert_eq!(specie.compartment().get().as_str(), "Intake");
        assert!(!specie.initial_amount().is_set());
        assert_eq!(specie.initial_concentration().get().unwrap(), 1051.0);
        assert!(!specie.substance_units().is_set());
        assert!(specie
            .has_only_substance_units()
            .get_checked()
            .unwrap()
            .is_none());
        assert!(specie.boundary_condition().get());
        assert!(specie.constant().get());
        assert!(!specie.conversion_factor().is_set());
        assert!(specie.annotation().is_set());

        let specie_empty = species.pop();
        assert_eq!(specie_empty.id().get().as_str(), "HDL");
        assert_eq!(specie_empty.compartment().get().as_str(), "Endothelium");
        assert!(!specie_empty.initial_amount().is_set());
        assert_eq!(specie_empty.initial_concentration().get().unwrap(), 0.0);
        assert!(!specie_empty.substance_units().is_set());
        assert!(specie_empty
            .has_only_substance_units()
            .get_checked()
            .unwrap()
            .is_none());
        assert!(!specie_empty.boundary_condition().get());
        assert!(!specie_empty.constant().get());
        assert!(!specie_empty.conversion_factor().is_set());
        assert!(!specie_empty.annotation().is_set());
    }

    #[test]
    pub fn test_parameters() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let parameters = model.parameters();
        assert!(parameters.is_set());

        let parameters = parameters.get().unwrap();
        assert!(!parameters.is_empty());
        assert_eq!(parameters.len(), 65);

        let parameter = parameters.get(0);
        assert!(parameter.constant().get());
        assert_eq!(parameter.id().get().as_str(), "alfa7");
        assert_eq!(parameter.name().get().unwrap(), "alfa7");
        assert_eq!(parameter.value().get().unwrap(), 2.8067);
        assert!(!parameter.units().is_set());
        assert!(!parameter.annotation().is_set());

        let parameter = parameters.pop();
        assert!(parameter.constant().get());
        assert_eq!(parameter.id().get().as_str(), "M");
        assert_eq!(parameter.name().get().unwrap(), "M");
        assert_eq!(parameter.value().get().unwrap(), 0.0155561);
        assert!(!parameter.units().is_set());
        assert!(!parameter.annotation().is_set());
    }

    #[test]
    pub fn test_rules() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let rules = model.rules();
        assert!(rules.is_set());

        let rules = rules.get().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules.len(), 9);

        match rules.get(0).cast() {
            RuleTypes::Algebraic(_) => panic!("The rule is algebraic."),
            Assignment(rule) => {
                assert_eq!(rule.variable().get().as_str(), "SUMRecTAINF");
                assert!(rule.math().is_set());
            }
            RuleTypes::Rate(_) => panic!("The rule is a rate rule."),
            RuleTypes::Other(_) => panic!("The rule is of a type this test does not expect."),
        };

        match rules.top().cast() {
            RuleTypes::Other(_) => panic!("The rule is of a type this test does not expect."),
            RuleTypes::Algebraic(_) => panic!("The rule is algebraic."),
            Assignment(rule) => {
                assert_eq!(rule.variable().get().as_str(), "SUMForFoam");
                assert!(rule.math().is_set());
            }
            RuleTypes::Rate(_) => panic!("The rule is a rate rule."),
        }
    }

    #[test]
    pub fn test_reactions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let reactions = model.reactions();
        assert!(reactions.is_set());

        let reactions = reactions.get().unwrap();
        assert!(!reactions.is_empty());
        assert_eq!(reactions.len(), 52);

        let reaction = reactions.get(0);
        assert_eq!(reaction.id().get().as_str(), "reaction_1");
        assert_eq!(reaction.meta_id().get().unwrap().as_str(), "COPASI41");
        assert_eq!(reaction.name().get().unwrap(), "Ingestion");
        assert!(!reaction.reversible().get());
        assert!(reaction.annotation().is_set());
        assert!(!reaction.compartment().is_set());

        let reactants = reaction.reactants();
        let products = reaction.products();
        let modifiers = reaction.modifiers();
        assert!(reactants.is_set());
        assert!(products.is_set());
        assert!(modifiers.is_set());

        let reactants = reactants.get().unwrap();
        let products = products.get().unwrap();
        let modifiers = modifiers.get().unwrap();
        assert!(!reactants.is_empty());
        assert_eq!(reactants.len(), 1);
        assert!(!products.is_empty());
        assert_eq!(products.len(), 1);
        assert!(!modifiers.is_empty());
        assert_eq!(modifiers.len(), 1);

        let reactant = reactants.pop();
        let product = products.pop();
        let modifier = modifiers.pop();
        assert_eq!(reactant.species().get().as_str(), "species_1");
        assert!(reactant.constant().get_checked().unwrap().is_none()); // doesn't conform to level3/version1/core
        assert_eq!(reactant.stoichiometry().get().unwrap(), 1.0);
        assert_eq!(product.species().get().as_str(), "species_2");
        assert!(product.constant().get_checked().unwrap().is_none()); // doesn't conform to level3/version1/core
        assert_eq!(product.stoichiometry().get().unwrap(), 1.0);
        assert_eq!(modifier.species().get().as_str(), "species_1");

        let kinetic_law = reaction.kinetic_law();
        assert!(kinetic_law.is_set());

        let kinetic_law = kinetic_law.get().unwrap();
        assert!(kinetic_law.math().is_set());

        let local_params = kinetic_law.local_parameters();
        assert!(local_params.is_set());

        let local_params = local_params.get().unwrap();
        assert!(!local_params.is_empty());
        assert_eq!(local_params.len(), 1);

        let param = local_params.pop();
        assert_eq!(param.id().get().as_str(), "k1");
        assert_eq!(param.name().get().unwrap(), "k1");
        assert_eq!(param.value().get().unwrap(), 1.0);
        assert!(!param.units().is_set());
    }

    #[test]
    pub fn test_initial_assignments() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let initial_assignments = model.initial_assignments();
        assert!(initial_assignments.is_set());

        let initial_assignments = initial_assignments.get().unwrap();
        assert!(!initial_assignments.is_empty());
        assert_eq!(initial_assignments.len(), 1);

        let single_assignment = initial_assignments.pop();
        assert_eq!(single_assignment.symbol().get().as_str(), "Susceptible");
        assert!(single_assignment.math().is_set());
    }

    #[test]
    pub fn test_events() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let events = model.events();
        assert!(events.is_set());

        let events = events.get().unwrap();
        assert!(!events.is_empty());
        assert_eq!(events.len(), 1);

        let event = events.top();
        assert_eq!(event.id().get().unwrap().as_str(), "Lockdown");
        assert_eq!(event.meta_id().get().unwrap().as_str(), "COPASI13");
        assert_eq!(event.name().get().unwrap(), "Lockdown");
        assert!(!event.priority().is_set());
        assert!(!event.delay().is_set());
        assert!(!event.use_values_from_trigger_time().get());

        let trigger = event.trigger();
        assert!(trigger.is_set());

        let trigger = trigger.get().unwrap();
        assert!(trigger.initial_value().get());
        assert!(!trigger.persistent().get());
        assert!(trigger.math().is_set());

        let assignments = event.event_assignments();
        assert!(assignments.is_set());

        let assignments = assignments.get().unwrap();
        assert!(!assignments.is_empty());
        assert_eq!(assignments.len(), 1);

        let evt_assignment = assignments.top();
        assert_eq!(evt_assignment.variable().get().as_str(), "epsilon");
        assert!(evt_assignment.math().is_set());
    }

    #[test]
    pub fn test_constraints() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let constraints = model.constraints();
        assert!(!constraints.is_set());
        constraints.ensure();
        assert!(constraints.is_set());
        let constraints = constraints.get().unwrap();

        assert!(constraints.is_empty());
        assert_eq!(constraints.len(), 0);
        assert!(!constraints.id().is_set());
        assert!(!constraints.name().is_set());
        assert!(!constraints.sbo_term().is_set());
        assert!(!constraints.annotation().is_set());
        assert!(!constraints.notes().is_set());
        assert!(!constraints.meta_id().is_set());
        assert_eq!(constraints.xml_element().namespace_url(), URL_SBML_CORE);

        let single = Constraint::default(model.xml_element().document());
        constraints.push(single.clone());
        assert!(!constraints.is_empty());
        assert_eq!(constraints.len(), 1);
        assert_eq!(
            constraints.top().xml_element().element(),
            single.xml_element().element()
        );

        let single = constraints.top();
        assert!(!single.math().is_set());
        assert!(!single.message().is_set());

        let math_el = Math::default(model.xml_element().document());
        let msg_el = XmlElement::new_qualified(model.xml_element().document(), "message", NS_HTML);
        msg_el.set_text_content("Warning: Unsatisfied constraint.");
        single.math().set(math_el);
        single.message().set(msg_el);
        assert!(single.math().is_set());
        assert!(single.message().is_set());
        assert!(single
            .message()
            .get()
            .unwrap()
            .text_content()
            .starts_with("Warning:"));
    }

    /// The identity of the document root accepts the namespace of *every* published SBML level and
    /// version, and nothing else.
    ///
    /// What this proves, precisely: `matches_element` is a membership test over the public constant
    /// `SBML_ROOT_NAMESPACES` — every URL the constant holds is accepted (so the constant cannot
    /// drift from the type test, which would fail here if the test compared against a single URL),
    /// and the two-sided part of the check is that a URL *outside* the constant is rejected. It does
    /// not, and cannot, enumerate the URLs a later level might define: what makes that a data change
    /// is that the implementation reads the constant instead of listing URLs itself.
    #[test]
    pub fn test_sbml_root_identity_is_a_membership_test_over_the_constant() {
        let document = || {
            Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8"?><probe/>"#)
                .expect("The probe document is well formed.")
        };
        // The namespaces are `&'static str` constants, so the closure takes one directly.
        let root = |namespace: &'static str| {
            XmlElement::new_qualified(document(), "sbml", ("", namespace))
        };

        assert!(SBML_ROOT_NAMESPACES.len() >= 8);
        for namespace in SBML_ROOT_NAMESPACES {
            let element = root(namespace);
            assert!(
                Sbml::matches_element(&element),
                "The root identity must accept the namespace {namespace}."
            );
            assert!(Sbml::try_from_element(element).is_some());
        }

        // The negative direction, so the membership claim is two-sided.
        for foreign in [
            "http://www.w3.org/1999/xhtml", // a namespace this crate knows, but not an SBML root
            "http://www.sbml.org/sbml/level9", // a level that does not exist
            "",                             // a document whose root declares no namespace
            "http://www.sbml.org/sbml/level2/version6", // not defined by any specification
        ] {
            let element = root(foreign);
            assert!(
                !Sbml::matches_element(&element),
                "The root identity must reject the namespace {foreign}."
            );
            assert!(Sbml::try_from_element(element).is_none());
        }

        // The name half of the identity is checked as well.
        let wrong_name = XmlElement::new_qualified(document(), "model", NS_SBML_CORE);
        assert!(!Sbml::matches_element(&wrong_name));

        // The namespace the crate *writes* with is the level 3 version 2 namespace, which is what
        // `ELEMENT_NAMESPACE` means for the root (a non-L3V2 document's root carries another one).
        assert_eq!(<Sbml as SbmlObject>::ELEMENT_NAMESPACE, NS_SBML_CORE);
        assert_eq!(<Sbml as SbmlObject>::ELEMENT_NAME, "sbml");
        assert!(<Sbml as SbmlObject>::CONSTRUCT.is_none());
    }

    /// The document *reader* stays namespace-agnostic: it dispatches on the local name, so a
    /// document whose root uses the namespace of another level is still recognized as an SBML
    /// document (and its issues are then reported as usual). Level dispatch is a separate concern;
    /// what this test pins is that adding the root identity did not tighten the reader.
    #[test]
    pub fn test_the_reader_still_accepts_other_sbml_levels() {
        for namespace in [
            "http://www.sbml.org/sbml/level1",
            "http://www.sbml.org/sbml/level2/version4",
        ] {
            let document = Sbml::read_str(&format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
                   <sbml xmlns="{namespace}" level="1" version="1"><model/></sbml>"#
            ))
            .expect("An older-level document is readable XML.");
            assert_eq!(document.xml_element().tag_name(), "sbml");
            assert!(
                Sbml::try_for_child_element(document.xml_element()).is_some(),
                "The reader must still recognize an {namespace} root."
            );
        }
    }

    #[test]
    pub fn test_package_manipulation() {
        let doc = Sbml::read_path("test-inputs/apoptosis_stable.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let layouts = model.layouts().get().unwrap();
        assert_eq!(layouts.len(), 1);
        println!("Read {} layouts", layouts.len());
        let mut issues = Vec::new();
        println!("{}", layouts.get(0).id().get());
        println!(
            "{}",
            layouts
                .get(0)
                .additional_graph_obj()
                .get()
                .unwrap()
                .get(0)
                .id()
                .get()
        );
        layouts
            .get(0)
            .validate(
                &mut issues,
                &mut Default::default(),
                &mut Default::default(),
                &mut crate::cancellation::CancellationCheck::new(),
            )
            .unwrap();
        println!("Issues: {:?}", issues);
        let layout = layouts.get(0);
        println!("{} {:?}", layout.id().get(), layout.name().get());
        let dimensions = layout.dimensions().get();
        println!("{} {}", dimensions.width().get(), dimensions.height().get());
        let additional_objects = layout.additional_graph_obj().get().unwrap();
        println!("Inside layout: {}", additional_objects.len());
        let obj = additional_objects.get(0);
        let obj = GeneralGlyph::try_cast_from_super(&obj).unwrap();
        assert_eq!(obj.reference().get().unwrap().as_str(), "csa2");
    }

    /// Creating a package element in a document that does not declare the package declares it:
    /// the conventional prefix of the package appears on the `<sbml>` element itself, together
    /// with the `<prefix>:required` attribute that the library writes as `false`.
    #[test]
    pub fn test_package_created() {
        let doc = Sbml::default();
        let model = doc.model().get_or_create();
        let _ = model.layouts().get_or_create();

        let root = doc.xml_element().element();
        let declarations = root.namespace_declarations();
        let declaration = declarations
            .into_iter()
            .find(|(_prefix, binding)| {
                binding
                    .as_ref()
                    .is_some_and(|it| it.uri() == URL_PACKAGE_LAYOUT)
            })
            .expect("The layout package namespace is declared on the <sbml> element.");
        assert_eq!(
            declaration.0.map(|it| it.as_str().to_string()),
            Some(NS_LAYOUT.0.to_string()),
            "The layout package must be declared with its conventional prefix."
        );

        let required = crate::constants::namespaces::qualified_name(NS_LAYOUT, "required").unwrap();
        let attributes = root.attributes();
        let attribute = attributes
            .iter()
            .find(|(name, _value)| **name == required)
            .expect("The layout package declaration must have a `required` attribute.");
        assert_eq!(attribute.0.to_string(), "layout:required");
        assert_eq!(attribute.1.as_str(), "false");
    }

    /// Pins the two package-declaration predicates: the conventional-prefix comparison that the
    /// package-aware validation of species and reactions uses, and the prefix-agnostic `is_err`
    /// check that the type check uses. A package counts as declared only when its namespace URL is
    /// bound on the `<sbml>` element *and* the corresponding `required` attribute is present.
    #[test]
    pub fn test_package_declaration_predicates() {
        let core = URL_SBML_CORE;
        let fbc = URL_PACKAGE_FBC;

        // (a) The conventional prefix, with the `required` attribute: declared.
        let doc = Sbml::read_str(&format!(
            r#"<sbml xmlns="{core}" xmlns:fbc="{fbc}" fbc:required="false"/>"#
        ))
        .unwrap();
        assert_eq!(doc.declared_package_prefix(NS_FBC), Ok("fbc".to_string()));

        // (b) The conventional prefix, without the `required` attribute: not declared.
        let doc = Sbml::read_str(&format!(r#"<sbml xmlns="{core}" xmlns:fbc="{fbc}"/>"#)).unwrap();
        assert!(doc.declared_package_prefix(NS_FBC).is_err());

        // (c) A non-conventional prefix, with the `required` attribute: declared, but the
        // conventional-prefix comparison does not match while the prefix-agnostic check does.
        let doc = Sbml::read_str(&format!(
            r#"<sbml xmlns="{core}" xmlns:fb="{fbc}" fb:required="false"/>"#
        ))
        .unwrap();
        assert_eq!(doc.declared_package_prefix(NS_FBC), Ok("fb".to_string()));
        assert_ne!(doc.declared_package_prefix(NS_FBC), Ok("fbc".to_string()));
        assert!(!doc.declared_package_prefix(NS_FBC).is_err());

        // (d) Not declared at all: not declared.
        let doc = Sbml::read_str(&format!(r#"<sbml xmlns="{core}"/>"#)).unwrap();
        assert!(doc.declared_package_prefix(NS_FBC).is_err());
    }

    /// Pins the package-name contract of the XML layer, which the whole-corpus gates cannot see:
    /// **the reader resolves a package name by its namespace URI** (the prefix a document happens
    /// to bind to that URI is irrelevant), while **the writer always introduces the conventional
    /// prefix of the package** on the `<sbml>` element.
    ///
    /// No document of the SBML test corpus (or of `test-inputs/`) binds a package URL under a
    /// non-conventional prefix, so `scripts/full_corpus_parity.sh` exercises neither half of this
    /// contract; this test is its tripwire. It pins the read/write asymmetry the writer is designed
    /// with - the reader accepts any prefix the document binds the package URL to, the writer always
    /// declares the conventional one - so a later change that makes the writer reuse the prefix found
    /// in the document fails here by design.
    #[test]
    pub fn test_package_read_path_is_prefix_agnostic() {
        let core = URL_SBML_CORE;
        let fbc = URL_PACKAGE_FBC;

        // The document binds the FBC namespace URL under `fb` and never under the conventional
        // `fbc`, so every FBC name below can only be found by comparing namespace URIs.
        let doc = Sbml::read_str(&format!(
            r#"<sbml xmlns="{core}" xmlns:fb="{fbc}" fb:required="false" level="3" version="1">
  <model>
    <listOfSpecies>
      <species id="s" fb:charge="2"/>
    </listOfSpecies>
  </model>
</sbml>"#
        ))
        .unwrap();

        // Negative controls: the document really carries no `fbc` prefix, and the declaration
        // predicate reports the prefix the document actually uses.
        let root = doc.xml_element().element();
        assert!(
            !root
                .namespace_declarations()
                .into_iter()
                .any(|(prefix, _)| prefix.as_ref().is_some_and(|it| it.as_str() == "fbc")),
            "The test document must not declare a conventional `fbc` prefix."
        );
        assert_eq!(doc.declared_package_prefix(NS_FBC), Ok("fb".to_string()));

        // READ: the package property resolves through the namespace URI alone.
        let model = doc.model().get().unwrap();
        let species = model.species().get().unwrap().get(0);
        let fbc_species = species.try_downcast::<FbcSpecies>().unwrap();
        assert_eq!(
            fbc_species.charge().get(),
            Some(2),
            "A package property must be found even when the document uses a different prefix."
        );

        // (2) Reading is *not* gated by the package declaration: a document that binds the URL
        // without a `required` attribute still resolves its package content. Whether the
        // declaration is complete is the validation layer's business, not the reader's.
        let undeclared = Sbml::read_str(&format!(
            r#"<sbml xmlns="{core}" xmlns:fb="{fbc}" level="3" version="1">
  <model>
    <listOfSpecies>
      <species id="s" fb:charge="4"/>
    </listOfSpecies>
  </model>
</sbml>"#
        ))
        .unwrap();
        assert!(undeclared.declared_package_prefix(NS_FBC).is_err());
        let undeclared_species = undeclared
            .model()
            .get()
            .unwrap()
            .species()
            .get()
            .unwrap()
            .get(0);
        assert_eq!(
            undeclared_species
                .try_downcast::<FbcSpecies>()
                .unwrap()
                .charge()
                .get(),
            Some(4),
            "The reader must resolve a package name by its namespace URI, not by the package \
             declaration."
        );

        // (3) WRITE: the library introduces the conventional prefix on the `<sbml>` element and
        // writes the property with it.
        fbc_species.charge().set_some(&3);

        let declaration = root
            .namespace_declarations()
            .into_iter()
            .find(|(prefix, _)| prefix.as_ref().is_some_and(|it| it.as_str() == "fbc"))
            .expect("The conventional `fbc` prefix must be declared on the `<sbml>` element.");
        assert_eq!(
            declaration.1.as_ref().map(|it| it.uri()),
            Some(fbc),
            "The `fbc` prefix must be bound to the FBC namespace URL."
        );
        // The declaration the document already had is added to, not rewritten.
        assert!(
            root.namespace_declarations()
                .into_iter()
                .any(|(prefix, _)| prefix.as_ref().is_some_and(|it| it.as_str() == "fb")),
            "The `fb` declaration of the document must be preserved."
        );

        let required = crate::constants::namespaces::qualified_name(NS_FBC, "required").unwrap();
        assert_eq!(
            root.attributes().get(&required).map(|it| it.as_str()),
            Some("false"),
            "The introduced declaration must carry `fbc:required=\"false\"`."
        );

        let xml = doc.to_xml_string().unwrap();
        assert!(
            xml.contains("fbc:charge=\"3\""),
            "The written property must use the conventional prefix: {xml}"
        );
        let reread = Sbml::read_str(xml.as_str()).unwrap();
        let reread_species = reread
            .model()
            .get()
            .unwrap()
            .species()
            .get()
            .unwrap()
            .get(0);
        assert_eq!(
            reread_species
                .try_downcast::<FbcSpecies>()
                .unwrap()
                .charge()
                .get(),
            Some(3),
            "The written value must survive a serialization round trip."
        );
    }

    #[test]
    pub fn test_element_search() {
        let doc = Sbml::read_path("test-inputs/apoptosis_stable.sbml").unwrap();

        let model = doc.model().get().unwrap();
        let layouts = model.layouts().get().unwrap();
        let layout = layouts.get(0);

        let csa_object = layout.find_element_by_sid(&sid("csa2"));
        assert!(csa_object.is_some());
        let csa_glyph = layout.find_by_sid::<GeneralGlyph>(&sid("csa2_glyph"));
        assert!(csa_glyph.is_some());
    }
}
