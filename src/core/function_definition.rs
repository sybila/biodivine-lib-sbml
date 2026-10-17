use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::math::Math;
use crate::xml::{Document, OptionalChild, XmlDefault, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "functionDefinition", namespace = NS_SBML_CORE)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    /// Try to find an instance of a [FunctionDefinition] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as one of its transitive parents is a
    /// [FunctionDefinition] element. If this is not satisfied, the method returns `None`.
    pub fn for_child_element(child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(child, "functionDefinition")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

impl XmlDefault for FunctionDefinition {
    fn default(document: Document) -> Self {
        FunctionDefinition::new_empty(document)
    }
}
