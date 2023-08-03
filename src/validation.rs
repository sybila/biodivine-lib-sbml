use xml_doc::Element;
use crate::SbmlDocument;

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
    fn validate(&self, document: &mut SbmlDocument, results: &mut Vec<SbmlIssue>);

}

pub struct SbmlIssue {
    /// Refers to the "raw" XML element where the issue occurred.
    pub element: Element,
    pub severity: SbmlIssueSeverity,
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