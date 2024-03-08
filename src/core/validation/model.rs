use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::core::{AbstractRule, Model, SBase, UnitDefinition};
use crate::core::RuleTypes::{Algebraic, Assignment};
use crate::core::RuleTypes::Rate;
use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309,
    apply_rule_10310, apply_rule_10311, apply_rule_10312, apply_rule_10313, apply_rule_10401,
    apply_rule_10402, apply_rule_10404, sanity_check, sanity_check_of_list,
    SanityCheckable, SbmlValidable, validate_list_of_objects,
};
use crate::core::validation::model::VertexType::{EQUATION, VARIABLE};
use crate::SbmlIssue;
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlElement, XmlProperty, XmlWrapper,
};

impl SbmlValidable for Model {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        self.apply_rule_10311(xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);
        self.apply_rule_10313(xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
            apply_rule_10402(&annotation, issues);
            apply_rule_10404(xml_element, issues);
        }
        if let Some(list_of_function_definition) = self.function_definitions().get() {
            validate_list_of_objects(&list_of_function_definition, issues, identifiers, meta_ids);
        }
        if let Some(list_of_unit_definitions) = self.unit_definitions().get() {
            validate_list_of_objects(&list_of_unit_definitions, issues, identifiers, meta_ids);
            UnitDefinition::apply_rule_10302(&list_of_unit_definitions, issues);
        }
        if let Some(list_of_compartments) = self.compartments().get() {
            validate_list_of_objects(&list_of_compartments, issues, identifiers, meta_ids);
        }
        if let Some(list_of_species) = self.species().get() {
            validate_list_of_objects(&list_of_species, issues, identifiers, meta_ids);
        }
        if let Some(list_of_parameters) = self.parameters().get() {
            validate_list_of_objects(&list_of_parameters, issues, identifiers, meta_ids);
        }
        if let Some(list_of_initial_assignment) = self.initial_assignments().get() {
            validate_list_of_objects(&list_of_initial_assignment, issues, identifiers, meta_ids);
        }
        if let Some(list_of_rules) = self.rules().get() {
            validate_list_of_objects(&list_of_rules, issues, identifiers, meta_ids);
            AbstractRule::apply_rule_10304(&list_of_rules, issues);
        }
        if let Some(list_of_constraint) = self.constraints().get() {
            validate_list_of_objects(&list_of_constraint, issues, identifiers, meta_ids);
        }
        if let Some(list_of_reactions) = self.reactions().get() {
            validate_list_of_objects(&list_of_reactions, issues, identifiers, meta_ids);
        }
        if let Some(list_of_events) = self.events().get() {
            validate_list_of_objects(&list_of_events, issues, identifiers, meta_ids);
        }
    }
}

impl SanityCheckable for Model {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);

        if let Some(list_of_function_definition) = self.function_definitions().get() {
            sanity_check_of_list(&list_of_function_definition, issues);
        }
        if let Some(list_of_unit_definitions) = self.unit_definitions().get() {
            sanity_check_of_list(&list_of_unit_definitions, issues);
        }
        if let Some(list_of_compartments) = self.compartments().get() {
            sanity_check_of_list(&list_of_compartments, issues);
        }
        if let Some(list_of_species) = self.species().get() {
            sanity_check_of_list(&list_of_species, issues);
        }
        if let Some(list_of_parameters) = self.parameters().get() {
            sanity_check_of_list(&list_of_parameters, issues);
        }
        if let Some(list_of_initial_assignment) = self.initial_assignments().get() {
            sanity_check_of_list(&list_of_initial_assignment, issues);
        }
        if let Some(list_of_rules) = self.rules().get() {
            sanity_check_of_list(&list_of_rules, issues);
        }
        if let Some(list_of_constraint) = self.constraints().get() {
            sanity_check_of_list(&list_of_constraint, issues);
        }
        if let Some(list_of_reactions) = self.reactions().get() {
            sanity_check_of_list(&list_of_reactions, issues);
        }
        if let Some(list_of_events) = self.events().get() {
            sanity_check_of_list(&list_of_events, issues);
        }
    }
}

impl Model {
    pub(crate) fn apply_rule_10311(&self, xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
        let sbstnc_units = self.substance_units();
        let volume_units = self.volume_units();
        let area_units = self.area_units();
        let length_units = self.length_units();
        let time_units = self.time_units();
        let extent_units = self.extent_units();

        apply_rule_10311(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);
        apply_rule_10311(volume_units.name(), volume_units.get(), xml_element, issues);
        apply_rule_10311(area_units.name(), area_units.get(), xml_element, issues);
        apply_rule_10311(length_units.name(), length_units.get(), xml_element, issues);
        apply_rule_10311(time_units.name(), time_units.get(), xml_element, issues);
        apply_rule_10311(extent_units.name(), extent_units.get(), xml_element, issues);
    }
    pub(crate) fn apply_rule_10313(&self, xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
        let sbstnc_units = self.substance_units();
        let volume_units = self.volume_units();
        let area_units = self.area_units();
        let length_units = self.length_units();
        let time_units = self.time_units();
        let extent_units = self.extent_units();

        apply_rule_10313(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);
        apply_rule_10313(volume_units.name(), volume_units.get(), xml_element, issues);
        apply_rule_10313(area_units.name(), area_units.get(), xml_element, issues);
        apply_rule_10313(length_units.name(), length_units.get(), xml_element, issues);
        apply_rule_10313(time_units.name(), time_units.get(), xml_element, issues);
        apply_rule_10313(extent_units.name(), extent_units.get(), xml_element, issues);
    }

    pub(crate) fn apply_rule_10701(&self, xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
        let mut bipartite_graph: HashMap<Vertex, Vec<Vertex>> = HashMap::new();

        self.load_vertices(bipartite_graph)
    }

    fn load_vertices(&self, mut graph: HashMap<Vertex, Vec<Vertex>>) {
        let mut vertices_equation: Vec<VertexKey> = Vec::new();
        let mut vertices_variable: Vec<VertexKey> = Vec::new();

        self.get_vertices_species(&mut vertices_equation, &mut vertices_variable);
        self.get_vertices_rules(&mut vertices_equation);
        self.get_vertices_reactions(&mut vertices_equation, &mut vertices_variable);
        self.get_vertices_compartments(&mut vertices_variable);

        insert_vertices(&mut graph, vertices_equation, EQUATION);
        insert_vertices(&mut graph, vertices_variable, VARIABLE);
    }

    /// Performs the following:
    /// - get equation vertices as of kinetic law
    /// - also get variable vertices as of SpeciesReference objects (reactants and products of a Reaction)
    /// - also get variable vertices as of Reaction objects
    fn get_vertices_reactions(
        &self,
        vertices_equation: &mut Vec<VertexKey>,
        vertices_variable: &mut Vec<VertexKey>,
    ) {
        self.reactions().get().and_then(|reactions| {
            Some(reactions.iter().for_each(|reaction| {
                reaction
                    .kinetic_law()
                    .get()
                    .and_then(|kinetic_law| Some(vertices_equation.push(VertexKey::KINETIC_LAW)));
                reaction.reactants().get().and_then(|reactants| {
                    Some(
                        reactants
                            .iter()
                            .filter(|reactant| !reactant.constant().get())
                            .for_each(|_| vertices_variable.push(VertexKey::SPECIES_REFERENCE)),
                    )
                });
                reaction.products().get().and_then(|products| {
                    Some(
                        products
                            .iter()
                            .filter(|product| !product.constant().get())
                            .for_each(|_| vertices_variable.push(VertexKey::SPECIES_REFERENCE)),
                    )
                });
                vertices_variable.push(VertexKey::REACTION)
            }))
        });
    }

    /// Performs the following:
    /// - get equation vertices as of species having boundaryCondition=false AND constant=false AND
    ///   which are referenced as reactant or product in one or more Reaction objects that contain
    ///   KineticLaw objects
    /// - also get variable vertices as of species having constant=false
    fn get_vertices_species(
        &self,
        vertices_equation: &mut Vec<VertexKey>,
        vertices_variable: &mut Vec<VertexKey>,
    ) {
        self.species().get().and_then(|species| {
            Some(species.iter().for_each(|s| {
                if (!s.boundary_condition().get()
                    && !s.constant().get()
                    && s.is_referenced_by_reaction(&self))
                {
                    vertices_equation.push(VertexKey::SPECIES)
                }
                if (!s.constant().get()) {
                    vertices_variable.push(VertexKey::SPECIES)
                }
            }))
        });
    }

    /// Performs the following:
    /// - get equation vertices as of rules
    fn get_vertices_rules(&self, vertices_equation: &mut Vec<VertexKey>) {
        self.rules().get().and_then(|rules| {
            Some(rules.iter().for_each(|r| match r.cast() {
                Assignment(_) => vertices_equation.push(VertexKey::ASSIGNMENT_RULE),
                Rate(_) => vertices_equation.push(VertexKey::RATE_RULE),
                Algebraic(_) => vertices_equation.push(VertexKey::ALGEBRAIC_RULE),
                _ => (),
            }))
        });
    }

    /// Performs the following:
    /// - get variable vertices as of compartments having constant=false
    fn get_vertices_compartments(&self, vertices_variable: &mut Vec<VertexKey>) {
        self.compartments().get().and_then(|compartments| {
            Some(
                compartments
                    .iter()
                    .filter(|c| !c.constant().get())
                    .for_each(|compartment| vertices_variable.push(VertexKey::COMPARTMENT)),
            )
        });
    }

    /// Performs the following:
    /// - get variable vertices as of parameters having constant=false
    fn get_vertices_parameters(&self, vertices_variable: &mut Vec<VertexKey>) {
        self.parameters().get().and_then(|parameters| {
            Some(
                parameters
                    .iter()
                    .filter(|p| !p.constant().get())
                    .for_each(|p| vertices_variable.push(VertexKey::PARAMETER)),
            )
        });
    }
}

enum VertexKey {
    SPECIES,
    ASSIGNMENT_RULE,
    RATE_RULE,
    ALGEBRAIC_RULE,
    KINETIC_LAW,
    COMPARTMENT,
    PARAMETER,
    SPECIES_REFERENCE,
    REACTION,
}

#[derive(Clone, Copy)]
enum VertexType {
    EQUATION,
    VARIABLE,
}

struct Vertex {
    v_key: VertexKey,
    v_type: VertexType,
}

fn insert_vertices(
    graph: &mut HashMap<Vertex, Vec<Vertex>>,
    keys: Vec<VertexKey>,
    vertex_type: VertexType,
) {
    for key in keys {
        graph.insert(
            Vertex {
                v_key: key,
                v_type: vertex_type,
            },
            Vec::new(),
        );
    }
}
