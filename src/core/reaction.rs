use crate::core::sbase::SbmlUtils;
use crate::core::{Math, SBase};
use crate::xml::{OptionalChild, OptionalProperty, RequiredProperty, XmlElement, XmlList};
use macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Reaction(XmlElement);

impl Reaction {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn reversible(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("reversible")
    }

    pub fn compartment(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("compartment")
    }

    pub fn reactants(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfReactants")
    }

    pub fn products(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfProducts")
    }

    pub fn modifiers(&self) -> OptionalChild<XmlList<ModifierSpeciesReference>> {
        self.optional_sbml_child("listOfModifiers")
    }

    pub fn kinetic_law(&self) -> OptionalChild<KineticLaw> {
        self.optional_sbml_child("kineticLaw")
    }
}

pub trait SimpleSpeciesReference: SBase {
    fn species(&self) -> RequiredProperty<String> {
        self.required_sbml_property("species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesReference(XmlElement);

impl SimpleSpeciesReference for SpeciesReference {}

impl SpeciesReference {
    pub fn stoichiometry(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("stoichiometry")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ModifierSpeciesReference(XmlElement);

impl SimpleSpeciesReference for ModifierSpeciesReference {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct KineticLaw(XmlElement);

impl KineticLaw {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    pub fn local_parameters(&self) -> OptionalChild<XmlList<LocalParameter>> {
        self.optional_sbml_child("listOfLocalParameters")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct LocalParameter(XmlElement);

impl LocalParameter {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("units")
    }
}
