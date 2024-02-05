use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement};
use macros::{SBase, XmlWrapper};

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

impl XmlDefault for FunctionDefinition {
    fn default(document: XmlDocument) -> Self {
        FunctionDefinition::new_empty(document, "functionDefinition")
    }
}
