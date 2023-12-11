use crate::core::sbase::SbmlUtils;
use crate::core::BaseUnit;
use crate::xml::{OptionalProperty, RequiredProperty, XmlElement};
use macros::{SBase, XmlWrapper};

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Parameter(XmlElement);

impl Parameter {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalProperty<BaseUnit> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}
