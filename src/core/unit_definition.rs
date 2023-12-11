use crate::core::sbase::SbmlUtils;
use crate::core::Unit;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement, XmlList};
use macros::{SBase, XmlWrapper};

/// Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct UnitDefinition(XmlElement);

impl UnitDefinition {
    pub fn units(&self) -> OptionalChild<XmlList<Unit>> {
        self.optional_sbml_child("listOfUnits")
    }
}

impl XmlDefault for UnitDefinition {
    fn default(document: XmlDocument) -> Self {
        UnitDefinition::new_empty(document, "unitDefinition")
    }
}
