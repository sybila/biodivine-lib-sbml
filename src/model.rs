use crate::xml::impl_xml_child::Child;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlWrapper};
use std::ops::{Deref, DerefMut};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new()
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

    /// return String or Double
    pub fn get_size(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "size")
            .map(|it| it.to_string())
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn get_units(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "units")
            .map(|it| it.to_string())
    }

    /// return String or Boolean ?
    pub fn get_constant(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "constant")
            .map(|it| it.to_string())
    }

    pub fn set_multiplier(&self, value: &String) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), "multiplier", value)
    }
}
