use std::ops::Deref;
use crate::xml::{XmlElement, XmlWrapper, XmlList};
use crate::sbase::SBaseDefault;

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug)]
pub struct SbmlModel {
    xml: XmlElement,
}

impl XmlWrapper for SbmlModel {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

/// Adds the default implementation of [SBase] to the [SbmlModel].
impl SBaseDefault for SbmlModel {}

impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel { xml }
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


#[derive(Clone, Debug)]
pub struct SbmlFunctionDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlFunctionDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlFunctionDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlFunctionDefinition { xml }
    }
}

#[derive(Clone, Debug)]
pub struct SbmlUnitDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlUnitDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlUnitDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlUnitDefinition { xml }
    }
}

impl SbmlUnitDefinition {
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

pub struct Unit {
    xml: XmlElement,
}

impl XmlWrapper for Unit {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for Unit {
    fn from(xml: XmlElement) -> Self {
        Unit { xml }
    }
}

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