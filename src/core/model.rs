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

    /// Returns a vector of [FunctionDefinition] identifiers (attribute **id**). Function definitions
    /// without IDs are not included in the output.
    pub(crate) fn function_definition_identifiers(&self) -> Vec<String> {
        if let Some(function_definitions) = self.function_definitions().get() {
            function_definitions
                .iter()
                .filter_map(|def| def.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Find a [FunctionDefinition] by its *id* and return a number of arguments this function expects.
    /// More precisely, find a number of **bvar** elements inside **lambda** inside **math** element of
    /// [FunctionDefinition]. If [FunctionDefinition] cannot be found or the is missing the appropriate
    /// math element, returns `None`.
    pub(crate) fn function_definition_arguments(&self, id: &str) -> Option<usize> {
        // Check that the list of a function definitions is present.
        let definitions = self.function_definitions().get()?;

        // And that we have found a function with the given id.
        let expected = Some(id.to_string());
        let function = definitions
            .iter()
            .find(|function| function.id().get() == expected)?;

        // And this function has its `math` element with a `lambda` child element specified.
        let doc = self.read_doc();
        let math = function.math().get()?;
        let lambda = math.raw_element().find(doc.deref(), "lambda")?;
        let lambda = XmlElement::new_raw(self.document(), lambda);

        // We then return the number of `bvar` child nodes in the lambda element.
        let count = lambda
            .child_elements_filtered(|it| it.name(doc.deref()) == "bvar")
            .len();

        Some(count)
    }

    /// Returns a vector of [UnitDefinition] identifiers (attribute **id**). Unit definitions
    /// without IDs are not included in the output.
    pub(crate) fn unit_definition_identifiers(&self) -> Vec<String> {
        if let Some(unit_definitions) = self.unit_definitions().get() {
            unit_definitions
                .iter()
                .filter_map(|unit| unit.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [LocalParameter] identifiers (attribute **id**).
    pub(crate) fn local_parameter_identifiers(&self) -> Vec<String> {
        let mut identifiers: Vec<String> = Vec::new();

        let Some(reactions) = self.reactions().get() else {
            return identifiers;
        };

        for reaction in reactions.iter() {
            let local_parameters = reaction
                .kinetic_law()
                .get()
                .and_then(|law| law.local_parameters().get());
            if let Some(local_parameters) = local_parameters {
                identifiers.extend(local_parameters.iter().map(|param| param.id().get()));
            }
        }

        identifiers
    }

    /// Returns a vector of [Species] identifiers (attribute **id**).
    pub(crate) fn species_identifiers(&self) -> Vec<String> {
        if let Some(species) = self.species().get() {
            species.iter().map(|species| species.id().get()).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [Compartment] identifiers (attribute **id**).
    pub(crate) fn compartment_identifiers(&self) -> Vec<String> {
        if let Some(compartment) = self.compartments().get() {
            compartment
                .iter()
                .map(|compartment| compartment.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [Parameter] identifiers (attribute **id**).
    pub(crate) fn parameter_identifiers(&self) -> Vec<String> {
        if let Some(parameters) = self.parameters().get() {
            parameters.iter().map(|param| param.id().get()).collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of [SpeciesReference] identifiers (attribute **id**). Unit definitions
    /// without IDs are not included in the output.
    pub(crate) fn species_reference_identifiers(&self) -> Vec<String> {
        let mut identifiers: Vec<String> = vec![];
        // If the list of reactions is present...
        if let Some(reactions) = self.reactions().get() {
            for reaction in reactions.as_vec() {
                // ...we extract identifiers of reactants and products.
                for list in &[reaction.reactants(), reaction.products()] {
                    if let Some(list) = list.get() {
                        identifiers.extend(list.iter().filter_map(|it| it.id().get()));
                    }
                }
            }
        }
        identifiers
    }

    /// Returns a vector of [FunctionDefinition] identifiers (attribute **id**).
    pub(crate) fn reaction_identifiers(&self) -> Vec<String> {
        if let Some(reactions) = self.reactions().get() {
            reactions
                .iter()
                .map(|reaction| reaction.id().get())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of all *variables* appearing in all [AssignmentRule] objects.
    pub(crate) fn assignment_rule_variables(&self) -> Vec<String> {
        if let Some(rules) = self.rules().get() {
            rules
                .iter()
                .filter_map(|rule| rule.try_downcast::<AssignmentRule>())
                .map(|assignment_rule| assignment_rule.variable().get())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of values from within the **ci** elements appearing in all [AlgebraicRule]
    /// objects in this model.
    pub(crate) fn algebraic_rule_ci_values(&self) -> Vec<String> {
        if let Some(rules) = self.rules().get() {
            let doc = self.read_doc();
            rules
                .iter()
                .filter_map(|rule| rule.try_downcast::<AlgebraicRule>())
                .filter_map(|algebraic_rule| algebraic_rule.math().get())
                .flat_map(|math| {
                    math.recursive_child_elements()
                        .into_iter()
                        .filter(|child| child.name(doc.deref()) == "ci")
                        .map(|ci| ci.text_content(doc.deref()))
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        }
    }

    /// Finds a species with the given *id*. If not found, returns `None`.
    pub(crate) fn find_species(&self, id: &str) -> Option<Species> {
        if let Some(species) = self.species().get() {
            species.iter().find(|species| species.id().get() == id)
        } else {
            None
        }
    }

    /// Finds a compartment with the given *id*. If not found, returns `None`.
    pub(crate) fn find_compartment(&self, id: &str) -> Option<Compartment> {
        if let Some(compartments) = self.compartments().get() {
            compartments
                .iter()
                .find(|compartment| compartment.id().get() == id)
        } else {
            None
        }
    }
}
