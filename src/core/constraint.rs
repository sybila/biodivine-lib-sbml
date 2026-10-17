use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{Document, OptionalChild, XmlDefault, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "constraint", namespace = NS_SBML_CORE)]
pub struct Constraint(XmlElement);

impl XmlDefault for Constraint {
    fn default(document: Document) -> Self {
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
