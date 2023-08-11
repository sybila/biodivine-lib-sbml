use crate::xml::impl_xml_child::Child;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlWrapper};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
    }

    pub fn function_definitions(&self) -> Child<XmlList<SbmlFunctionDefinition>> {
        Child::new(self.as_xml(), "listOfFunctionDefinitions")
    }

    pub fn unit_definitions(&self) -> Child<XmlList<SbmlUnitDefinition>> {
        Child::new(self.as_xml(), "listOfUnitDefinitions")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> Child<XmlList<Unit>> {
        Child::new(self.as_xml(), "listOfUnits")
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
