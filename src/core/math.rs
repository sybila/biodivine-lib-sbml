use crate::constants::namespaces::NS_MATHML;
use crate::xml::{XmlDefault, XmlDocument, XmlElement, XmlWrapper};
use sbml_macros::XmlWrapper;

/// A [Math] element represents an [XmlElement] related to MathML which is
/// separated from SBML specification.
#[derive(Clone, Debug, XmlWrapper)]
pub struct Math(XmlElement);

impl XmlDefault for Math {
    fn default(document: XmlDocument) -> Self {
        unsafe { Math::unchecked_cast(XmlElement::new_quantified(document, "math", NS_MATHML)) }
    }
}
