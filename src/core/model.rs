use crate::core::sbase::SbmlUtils;
use crate::core::{
    AbstractRule, Compartment, Constraint, Event, FunctionDefinition, InitialAssignment, Parameter,
    Reaction, Species, UnitDefinition,
};
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement, XmlList};
use macros::{SBase, XmlWrapper};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Model(XmlElement);

impl XmlDefault for Model {
    fn default(document: XmlDocument) -> Self {
        Model::new_empty(document, "model")
    }
}

/// Public functions to manipulate with the contents of SBML [Model]
/// i.e., optional lists inside SBML model
impl Model {
    pub fn function_definitions(&self) -> OptionalChild<XmlList<FunctionDefinition>> {
        self.optional_sbml_child("listOfFunctionDefinitions")
    }

    pub fn unit_definitions(&self) -> OptionalChild<XmlList<UnitDefinition>> {
        self.optional_sbml_child("listOfUnitDefinitions")
    }

    pub fn compartments(&self) -> OptionalChild<XmlList<Compartment>> {
        self.optional_sbml_child("listOfCompartments")
    }

    pub fn species(&self) -> OptionalChild<XmlList<Species>> {
        self.optional_sbml_child("listOfSpecies")
    }

    pub fn parameters(&self) -> OptionalChild<XmlList<Parameter>> {
        self.optional_sbml_child("listOfParameters")
    }

    pub fn initial_assignments(&self) -> OptionalChild<XmlList<InitialAssignment>> {
        self.optional_sbml_child("listOfInitialAssignments")
    }

    pub fn rules(&self) -> OptionalChild<XmlList<AbstractRule>> {
        self.optional_sbml_child("listOfRules")
    }

    pub fn constraints(&self) -> OptionalChild<XmlList<Constraint>> {
        self.optional_sbml_child("listOfConstraints")
    }

    pub fn reactions(&self) -> OptionalChild<XmlList<Reaction>> {
        self.optional_sbml_child("listOfReactions")
    }

    pub fn events(&self) -> OptionalChild<XmlList<Event>> {
        self.optional_sbml_child("listOfEvents")
    }
}
