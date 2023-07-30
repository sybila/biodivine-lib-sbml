use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlChild, XmlWrapper};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

#[derive(XmlChild)]
#[child(listOfFunctionDefinitions : XmlList<SbmlFunctionDefinition>)]
pub struct ListOfFunctionDefinitions<'a>(&'a XmlElement);

#[derive(XmlChild)]
#[child(listOfUnitDefinitions : XmlList<SbmlUnitDefinition>)]
pub struct ListOfUnitDefinitions<'a>(&'a XmlElement);

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new() 
impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
    }

    pub fn function_definitions(&self) -> ListOfFunctionDefinitions {
        ListOfFunctionDefinitions::for_element(self.as_xml())
    }

    pub fn unit_definitions(&self) -> ListOfUnitDefinitions {
        ListOfUnitDefinitions::for_element(self.as_xml())
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

#[derive(XmlChild)]
#[child(listOfUnits : XmlList<Unit>)]
pub struct ListOfUnits<'a>(&'a XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> ListOfUnits {
        ListOfUnits::for_element(self.as_xml())
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
