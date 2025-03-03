use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalProperty, RequiredProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Species(XmlElement);

impl Species {
    pub fn new(document: XmlDocument, id: &SId, compartment: &SId) -> Self {
        let obj = Species::new_empty(document, "species");
        obj.id().set(id);
        obj.compartment().set(compartment);
        obj.has_only_substance_units().set(&false);
        obj.boundary_condition().set(&true);
        obj.constant().set(&true);
        obj
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }

    pub fn compartment(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("compartment")
    }

    pub fn initial_amount(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialAmount")
    }

    pub fn initial_concentration(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    pub fn substance_units(&self) -> OptionalProperty<SId> {
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

    pub fn conversion_factor(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("conversionFactor")
    }
}
