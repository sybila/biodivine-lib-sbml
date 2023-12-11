use crate::core::sbase::SbmlUtils;
use crate::core::BaseUnit;
use crate::xml::{OptionalProperty, RequiredProperty, XmlElement};
use macros::{SBase, XmlWrapper};

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Species(XmlElement);

impl Species {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn compartment(&self) -> RequiredProperty<String> {
        self.required_sbml_property("compartment")
    }

    pub fn initial_amount(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialAmount")
    }

    pub fn initial_concentration(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    pub fn substance_units(&self) -> OptionalProperty<BaseUnit> {
        self.optional_sbml_property("substanceUnits")
    }

    pub fn has_only_substance_units(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("hasOnlySubstanceUnits")
    }

    pub fn boundary_condition(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("boundaryCondition")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }

    pub fn conversion_factor(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("conversionFactor")
    }
}
