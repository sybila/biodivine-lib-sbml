use crate::core::sbase::SbmlUtils;
use crate::core::{
    AbstractRule, AlgebraicRule, AssignmentRule, Compartment, Constraint, Event,
    FunctionDefinition, InitialAssignment, Parameter, Reaction, Rule, SBase, Species,
    UnitDefinition,
};
use crate::xml::{
    OptionalChild, OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlDefault,
    XmlDocument, XmlElement, XmlList, XmlSupertype, XmlWrapper,
};
use macros::{SBase, XmlWrapper};

use std::ops::Deref;
use xml_doc::Element;

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
    /// Try to find an instance of a [Model] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as it appears in an SBML model (i.e. one of
    /// its transitive parents is a [Model] element). If this is not satisfied, the method
    /// returns `None`.
    pub fn for_child_element(doc: XmlDocument, child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(doc, child, "model")
    }

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

    /// Returns a vector of [FunctionDefinition]s' identifiers (attribute **id**). If the identifier is not set,
    /// it is not included in the output.
    pub(crate) fn function_definition_identifiers(&self) -> Vec<String> {
        if let Some(function_definitions) = self.function_definitions().get() {
            function_definitions
                .as_vec()
                .iter()
                .filter_map(|def| def.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Find a [FunctionDefinition] by its *id* and return a number of arguments this function expects.
    /// More precisely, find a number of **bvar** elements inside **lambda** inside **math** element of
    /// [FunctionDefinition]. If [FunctionDefinition] cannot be found, returns 0.
    pub(crate) fn function_definition_arguments(&self, id: &str) -> i32 {
        // if list of function definitions is present
        if let Some(function_definitions) = self.function_definitions().get() {
            let function_definitions = function_definitions.as_vec();
            // and we have found a function with given id
            if let Some(function) = function_definitions
                .iter()
                .find(|function| function.id().get() == Some(id.to_string()))
            {
                // and this function has its math element specified
                if let Some(math) = function.math().get() {
                    let doc = self.read_doc();
                    // and a lambda element within math is present
                    if let Some(lambda) = math.raw_element().find(doc.deref(), "lambda") {
                        // we return a number of bvar elements
                        return lambda
                            .child_elements(doc.deref())
                            .iter()
                            .filter(|child| child.name(doc.deref()) == "bvar")
                            .collect::<Vec<&Element>>()
                            .len() as i32;
                    }
                }
            }
        }
        0
    }

    /// Returns a vector of [UnitDefinition]s' identifiers (attribute **id**). If the identifier is not set,
    /// it is not included in the output.
    pub(crate) fn unit_definition_identifiers(&self) -> Vec<String> {
        if let Some(unit_definitions) = self.unit_definitions().get() {
            unit_definitions
                .as_vec()
                .iter()
                .filter_map(|unit| unit.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [LocalParameter]s' identifiers (attribute **id**).
    pub(crate) fn local_parameter_identifiers(&self) -> Vec<String> {
        let mut identifiers: Vec<String> = vec![];

        if let Some(reactions) = self.reactions().get() {
            for reaction in reactions.as_vec() {
                if let Some(kinetic_law) = reaction.kinetic_law().get() {
                    if let Some(local_params) = kinetic_law.local_parameters().get() {
                        let mut param_ids = local_params
                            .as_vec()
                            .iter()
                            .map(|param| param.id().get())
                            .collect::<Vec<String>>();
                        identifiers.append(&mut param_ids);
                    }
                }
            }
        }
        identifiers
    }

    /// Returns a vector of all [Species]' identifiers (attribute **id**).
    pub(crate) fn species_identifiers(&self) -> Vec<String> {
        if let Some(species) = self.species().get() {
            species
                .as_vec()
                .iter()
                .map(|species| species.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [Compartment]s' identifiers (attribute **id**).
    pub(crate) fn compartment_identifiers(&self) -> Vec<String> {
        if let Some(compartment) = self.compartments().get() {
            compartment
                .as_vec()
                .iter()
                .map(|compartment| compartment.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [Parameter]s' identifiers (attribute **id**).
    pub(crate) fn parameter_identifiers(&self) -> Vec<String> {
        if let Some(parameters) = self.parameters().get() {
            parameters
                .as_vec()
                .iter()
                .map(|param| param.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [SpeciesReference](crate::core::SpeciesReference)' identifiers (attribute **id**).
    /// If the identifier is not set, it is not included in the output.
    pub(crate) fn species_reference_identifiers(&self) -> Vec<String> {
        let mut identifiers: Vec<String> = vec![];
        // if list of reactions is present
        if let Some(reactions) = self.reactions().get() {
            for reaction in reactions.as_vec() {
                // we extract identifiers of reactants
                let mut reactants = match reaction.reactants().get() {
                    Some(reactants) => reactants
                        .as_vec()
                        .iter()
                        .filter_map(|reactant| reactant.id().get())
                        .collect::<Vec<String>>(),
                    None => vec![],
                };
                // and product identifiers as well
                let mut products = match reaction.products().get() {
                    Some(products) => products
                        .as_vec()
                        .iter()
                        .filter_map(|product| product.id().get())
                        .collect::<Vec<String>>(),
                    None => vec![],
                };
                // and then we include results in the output
                identifiers.append(&mut reactants);
                identifiers.append(&mut products);
            }
        }
        identifiers
    }

    /// Returns a vector of all [Reaction]s' identifiers (attribute **id**).
    pub(crate) fn reaction_identifiers(&self) -> Vec<String> {
        if let Some(reactions) = self.reactions().get() {
            reactions
                .as_vec()
                .iter()
                .map(|reaction| reaction.id().get())
                .collect::<Vec<String>>()
        } else {
            vec![]
        }
    }

    /// Returns a vector of *variables* of all [AssignmentRule]s.
    pub(crate) fn assignment_rule_variables(&self) -> Vec<String> {
        if let Some(rules) = self.rules().get() {
            return rules
                .as_vec()
                .iter()
                .filter_map(|rule| rule.try_downcast::<AssignmentRule>())
                .map(|assignment_rule| assignment_rule.variable().get())
                .collect::<Vec<String>>();
        }
        vec![]
    }

    /// Returns a vector of values from within **ci** element.
    pub(crate) fn algebraic_rule_ci_values(&self) -> Vec<String> {
        if let Some(rules) = self.rules().get() {
            let doc = self.read_doc();
            return rules
                .as_vec()
                .iter()
                .filter_map(|rule| rule.try_downcast::<AlgebraicRule>())
                .filter_map(|algebraic_rule| algebraic_rule.math().get())
                .flat_map(|math| {
                    math.raw_element()
                        .child_elements_recursive(doc.deref())
                        .iter()
                        .filter(|child| child.name(doc.deref()) == "ci")
                        .map(|ci| ci.text_content(doc.deref()))
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<String>>();
        }
        vec![]
    }

    /// Finds a species with given *id*. If not found, returns None.
    pub(crate) fn find_species(&self, id: &str) -> Option<Species> {
        if let Some(species) = self.species().get() {
            species
                .as_vec()
                .iter()
                .find(|species| species.id().get() == id)
                .cloned()
        } else {
            None
        }
    }

    /// Finds a compartment with given *id*. If not found, returns None.
    pub(crate) fn find_compartment(&self, id: &str) -> Option<Compartment> {
        if let Some(compartments) = self.compartments().get() {
            compartments
                .as_vec()
                .iter()
                .find(|compartment| compartment.id().get() == id)
                .cloned()
        } else {
            None
        }
    }
}
