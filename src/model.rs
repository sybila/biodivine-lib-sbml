use crate::sbase::SBaseDefault;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::XmlWrapper;
use std::ops::Deref;

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlModel(XmlElement);

/// Adds the default implementation of [SBase] to the [SbmlModel].
impl SBaseDefault for SbmlModel {}

impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
    }
    pub fn get_function_definitions(&self) -> XmlList<SbmlFunctionDefinition> {
        let list_element = {
            let xml = self.read_doc();
            self.element()
                .find(xml.deref(), "listOfFunctionDefinitions")
                .unwrap()
        };
        XmlList::from(self.as_xml().derive(list_element))
    }

    pub fn get_unit_definitions(&self) -> XmlList<SbmlUnitDefinition> {
        let list = self.child_element("listOfUnitDefinitions");
        XmlList::from(self.as_xml().derive(list))
    }
}

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

#[derive(Clone, Debug, XmlWrapper)]
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
/// TODO: If I recall correctly, these should also implement SBase, but remove if that's not true.
impl SBaseDefault for SbmlFunctionDefinition {}
