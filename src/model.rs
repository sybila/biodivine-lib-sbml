use crate::xml::{OptionalChild, RequiredChild, XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlWrapper};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
    }

    pub fn function_definitions(&self) -> OptionalChild<XmlList<SbmlFunctionDefinition>> {
        OptionalChild::new(self.as_xml(), "listOfFunctionDefinitions")
    }

    pub fn unit_definitions(&self) -> RequiredChild<XmlList<SbmlUnitDefinition>> {
        RequiredChild::new(self.as_xml(), "listOfUnitDefinitions")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> RequiredChild<XmlList<Unit>> {
        RequiredChild::new(self.as_xml(), "listOfUnits")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Unit(XmlElement);

impl Unit {
    pub fn get_kind(&self) {
        todo!()
    }

    pub fn get_exponent(&self) {
        todo!()
    }

    pub fn get_scale(&self) {
        todo!()
    }

    pub fn get_multiplier(&self) {
        todo!()
    }
}
