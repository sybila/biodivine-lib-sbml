use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, RequiredProperty, RequiredXmlProperty, XmlDocument, XmlElement};
use macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn new(document: XmlDocument, symbol: &String) -> Self {
        let obj = InitialAssignment::new_empty(document, "initialAssignment");
        obj.symbol().set(symbol);
        obj
    }

    pub fn symbol(&self) -> RequiredProperty<String> {
        self.required_sbml_property("symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
