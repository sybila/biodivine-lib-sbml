use crate::xml::OptionalXmlChild;
use crate::Sbml;
use xml_doc::{Document, Element};

/// A trait implemented by SBML components that perform some form of validation.
pub trait SbmlValidate {
    /// Perform all validation steps required for `Self`.
    ///
    /// The method should put any issues into the `results` vector (this is more efficient
    /// than creating a new vector per element and then merging them all together).
    ///
    /// For any validation that needs to synchronize "globally" across the whole document,
    /// use the attached [SbmlDocument] object.
    ///
    /// For hierarchical components, it is the responsibility of the parent component to
    /// invoke the `validate` method on all of its child components.
    fn validate(&self, document: &mut Sbml, results: &mut Vec<SbmlIssue>);
}

pub struct SbmlIssue {
    /// Refers to the "raw" XML element where the issue occurred.
    pub element: Element,
    pub severity: SbmlIssueSeverity,
    pub rule: String,
    pub message: String,
}

pub enum SbmlIssueSeverity {
    /// An issue that makes the document impossible to read correctly (e.g. a function is
    /// used but not declared).
    Error,
    /// An issue that suggests a possible error but does not necessarily make the document
    /// invalid (e.g. a variable is declared but never used).
    Warning,
    /// A suggestion that would improve the document but does not represent a significant
    /// issue (e.g. an property is included when it does not have to be, or unknown tags
    /// or attributes are present in the document, e.g. due to the use of unofficial extensions).
    Info,
}

/// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
/// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
/// elements or attributes placed in an SBML namespace do not conform to the SBML
/// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
pub fn apply_rule_10102(doc: &Document, sbml: &Sbml, issues: &mut Vec<SbmlIssue>) {
    let rule_number = "10102".to_string();

    if doc.root_nodes().len() != 1 {
        issues.push(SbmlIssue {
            element: doc.container(),
            severity: SbmlIssueSeverity::Error,
            rule: rule_number.clone(),
            message: "The document contains multiple root nodes.".to_string(),
        })
    }

    if let Some(root_element) = doc.root_element() {
        if root_element.name(doc) == "sbml" {
            if let Some(model) = sbml.model().get() {
                model.apply_rule_10102(issues);
            }
        } else {
            issues.push(SbmlIssue {
                element: root_element,
                severity: SbmlIssueSeverity::Error,
                rule: rule_number,
                message: "Root element is invalid".to_string(),
            })
        }
    }
}
