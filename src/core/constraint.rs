use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Constraint(XmlElement);

impl XmlDefault for Constraint {
    fn default(document: XmlDocument) -> Self {
        Constraint::new_empty(document, "constraint")
    }
}

impl Constraint {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    pub fn message(&self) -> OptionalChild<XmlElement> {
        self.optional_html_child("message")
    }
}
