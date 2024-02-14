use crate::constants::namespaces::URL_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::{
    AbstractRule, Compartment, Constraint, Event, FunctionDefinition, InitialAssignment, Parameter,
    Reaction, SBase, Species, UnitDefinition,
};
use crate::xml::{
    OptionalChild, OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlDefault,
    XmlDocument, XmlElement, XmlList, XmlWrapper,
};
use macros::{SBase, XmlWrapper};
use std::ops::Deref;
use xml_doc::{Document, Element};

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
        let parent = {
            let read_doc = doc.read().unwrap();
            fn is_model(doc: &Document, e: Element) -> bool {
                let name = e.name(doc);
                let Some(namespace) = e.namespace(doc) else {
                    return false;
                };

                name == "model" && namespace == URL_SBML_CORE
            }

            let mut parent = child.raw_element();
            while !is_model(read_doc.deref(), parent) {
                let Some(node) = parent.parent(read_doc.deref()) else {
                    return None;
                };
                parent = node;
            }

            parent
        };
        let model = XmlElement::new_raw(doc, parent);
        // Safe because we checked that the element has the correct tag name and namespace.
        Some(unsafe { Model::unchecked_cast(model) })
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
        let function_definitions = self.function_definitions();

        if function_definitions.is_set() {
            function_definitions
                .get()
                .unwrap()
                .as_vec()
                .iter()
                .filter_map(|def| def.id().get())
                .collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [LocalParameter]s' identifiers (attribute **id**).
    pub(crate) fn local_parameter_identifiers(&self) -> Vec<String> {
        let reactions = self.reactions();
        let mut vec: Vec<String> = vec![];

        if reactions.is_set() {
            for reaction in reactions.get().unwrap().as_vec() {
                let kinetic_law = reaction.kinetic_law();

                if kinetic_law.is_set() {
                    let kinetic_law = kinetic_law.get().unwrap();
                    let local_params = kinetic_law.local_parameters();

                    if local_params.is_set() {
                        let mut param_ids = local_params
                            .get()
                            .unwrap()
                            .as_vec()
                            .iter()
                            .map(|param| param.id().get())
                            .collect::<Vec<String>>();
                        vec.append(&mut param_ids);
                    }
                }
            }
        }
        vec
    }

    /// Returns a vector of all [Species]' identifiers (attribute **id**).
    pub(crate) fn species_identifiers(&self) -> Vec<String> {
        let species = self.species();

        if species.is_set() {
            species
                .get()
                .unwrap()
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
        let compartment = self.compartments();

        if compartment.is_set() {
            compartment
                .get()
                .unwrap()
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
        let parameters = self.parameters();

        if parameters.is_set() {
            parameters
                .get()
                .unwrap()
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
        let reactions = self.reactions();
        let mut identifiers: Vec<String> = vec![];

        if reactions.is_set() {
            for reaction in reactions.get().unwrap().as_vec() {
                let mut reactants = match reaction.reactants().get() {
                    Some(reactants) => reactants
                        .as_vec()
                        .iter()
                        .filter_map(|reactant| reactant.id().get())
                        .collect::<Vec<String>>(),
                    None => vec![],
                };

                let mut products = match reaction.products().get() {
                    Some(products) => products
                        .as_vec()
                        .iter()
                        .filter_map(|product| product.id().get())
                        .collect::<Vec<String>>(),
                    None => vec![],
                };

                identifiers.append(&mut reactants);
                identifiers.append(&mut products);
            }
            identifiers
        } else {
            vec![]
        }
    }

    /// Returns a vector of all [Reaction]s' identifiers (attribute **id**).
    pub(crate) fn reaction_identifiers(&self) -> Vec<String> {
        let reactions = self.reactions();

        if reactions.is_set() {
            reactions
                .get()
                .unwrap()
                .as_vec()
                .iter()
                .map(|reaction| reaction.id().get())
                .collect::<Vec<String>>()
        } else {
            vec![]
        }
    }
}
