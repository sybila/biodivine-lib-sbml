use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
    XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "species", namespace = NS_SBML_CORE)]
pub struct Species(XmlElement);

impl XmlSupertype for Species {}

impl Species {
    /// Creates a new `<species>` element with the given `id`, `compartment`.
    pub fn new(document: Document, id: &SId, compartment: &SId) -> Self {
        let obj = Species::new_empty(document);
        obj.id().set(id);
        obj.compartment().set(compartment);
        obj.has_only_substance_units().set(&false);
        obj.boundary_condition().set(&true);
        obj.constant().set(&true);
        obj
    }

    /// The `id` attribute of this `<species>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("id")
    }

    /// The `compartment` attribute of this `<species>` element.
    pub fn compartment(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("compartment")
    }

    /// The `initialAmount` attribute of this `<species>` element.
    pub fn initial_amount(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("initialAmount")
    }

    /// The `initialConcentration` attribute of this `<species>` element.
    pub fn initial_concentration(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    /// The `substanceUnits` attribute of this `<species>` element.
    pub fn substance_units(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("substanceUnits")
    }

    /// The `hasOnlySubstanceUnits` attribute of this `<species>` element.
    pub fn has_only_substance_units(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("hasOnlySubstanceUnits")
    }

    /// The `boundaryCondition` attribute of this `<species>` element.
    pub fn boundary_condition(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("boundaryCondition")
    }

    /// The `constant` attribute of this `<species>` element.
    pub fn constant(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("constant")
    }

    /// The `conversionFactor` attribute of this `<species>` element.
    pub fn conversion_factor(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("conversionFactor")
    }
}
