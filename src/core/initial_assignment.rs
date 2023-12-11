use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, RequiredProperty, XmlElement};
use macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn symbol(&self) -> RequiredProperty<String> {
        self.required_sbml_property("symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
