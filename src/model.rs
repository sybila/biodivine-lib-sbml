use crate::xml::{
    OptionalChild, RequiredChild, XmlDefault, XmlDocument, XmlElement, XmlList, XmlWrapper,
};
use crate::{NS_SBML_CORE, URL_SBML_CORE};
use macros::{SBase, XmlWrapper};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

impl SbmlModel {
    pub fn function_definitions(&self) -> OptionalChild<XmlList<SbmlFunctionDefinition>> {
        OptionalChild::new(self.as_xml(), "listOfFunctionDefinitions", URL_SBML_CORE)
    }

    pub fn unit_definitions(&self) -> OptionalChild<XmlList<SbmlUnitDefinition>> {
        OptionalChild::new(self.as_xml(), "listOfUnitDefinitions", URL_SBML_CORE)
    }
}

impl XmlDefault for SbmlModel {
    fn default(document: XmlDocument) -> Self {
        unsafe {
            SbmlModel::unchecked_cast(XmlElement::new_quantified(document, "model", NS_SBML_CORE))
        }
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> RequiredChild<XmlList<Unit>> {
        RequiredChild::new(self.as_xml(), "listOfUnits", URL_SBML_CORE)
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
