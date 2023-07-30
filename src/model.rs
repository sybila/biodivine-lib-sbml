use crate::sbase::SBaseDefault;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::XmlWrapper;
use std::ops::Deref;

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlModel(XmlElement);

/// Adds the default implementation of [SBase] to the [SbmlModel].
/// Allows to get/set id, name, etc. of [SbmlModel]
impl SBaseDefault for SbmlModel {}

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new() 
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
        // possible better-readability & no copy-paste approach
        let list = self.child_element("listOfUnitDefinitions");
        XmlList::from(self.as_xml().derive(list))
    }
}

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    /// Get inner list of [Unit] elements.
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

#[derive(Clone, Debug, XmlWrapper)]
pub struct Unit(XmlElement);

impl Unit {
    pub fn get_kind(&self) -> String {
        let doc = self.read_doc();
        self.element().attribute(doc.deref(), "kind").unwrap().to_string()
    }

    pub fn get_exponent(&self) -> String {
        let doc = self.read_doc();
        self.element().attribute(doc.deref(), "exponent").unwrap().to_string()
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
