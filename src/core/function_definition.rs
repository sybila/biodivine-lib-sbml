use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement};
use macros::{SBase, XmlWrapper};

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    /// Try to find an instance of a [FunctionDefinition] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as one of its transitive parents is a
    /// [FunctionDefinition] element. If this is not satisfied, the method returns `None`.
    pub fn for_child_element(doc: XmlDocument, child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(doc, child, "functionDefinition")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

impl XmlDefault for FunctionDefinition {
    fn default(document: XmlDocument) -> Self {
        FunctionDefinition::new_empty(document, "functionDefinition")
    }
}
