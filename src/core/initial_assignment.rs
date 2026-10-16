use crate::core::sbase::SbmlUtils;
use crate::core::{Math, SId};
use crate::xml::{Document, OptionalChild, RequiredSbmlProperty, RequiredXmlProperty, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn new(document: Document, symbol: &SId) -> Self {
        let obj = InitialAssignment::new_empty(document, "initialAssignment");
        obj.symbol().set(symbol);
        obj
    }

    pub fn symbol(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
