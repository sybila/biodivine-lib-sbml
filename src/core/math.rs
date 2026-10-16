use crate::constants::namespaces::NS_MATHML;
use crate::xml::{Document, XmlDefault, XmlElement};
use sbml_macros::XmlWrapper;

/// A [Math] element represents an [XmlElement] related to MathML which is
/// separated from SBML specification.
#[derive(Clone, Debug, XmlWrapper)]
pub struct Math(XmlElement);

impl XmlDefault for Math {
    fn default(document: Document) -> Self {
        Math::from(XmlElement::new_qualified(document, "math", NS_MATHML))
    }
}
