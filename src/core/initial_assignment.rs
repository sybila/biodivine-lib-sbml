use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::math::Math;
use crate::xml::{Document, OptionalChild, RequiredSbmlProperty, RequiredXmlProperty, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "initialAssignment", namespace = NS_SBML_CORE)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn new(document: Document, symbol: &SId) -> Self {
        let obj = InitialAssignment::new_empty(document);
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
