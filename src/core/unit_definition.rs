use crate::core::sbase::SbmlUtils;
use crate::core::Unit;
use crate::xml::{Document, OptionalSbmlChild, XmlDefault, XmlElement, XmlList};
use sbml_macros::{SBase, XmlWrapper};

/// Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct UnitDefinition(XmlElement);

impl UnitDefinition {
    pub fn units(&self) -> OptionalSbmlChild<XmlList<Unit>> {
        self.optional_sbml_child("listOfUnits")
    }
}

impl XmlDefault for UnitDefinition {
    fn default(document: Document) -> Self {
        UnitDefinition::new_empty(document, "unitDefinition")
    }
}
