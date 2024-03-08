use crate::core::validation::model::VertexType::{Equation, Variable};
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309, apply_rule_10310,
    apply_rule_10311, apply_rule_10312, apply_rule_10313, apply_rule_10401, apply_rule_10402,
    validate_list_of_objects, SbmlValidable,
};
use crate::core::RuleTypes::{Algebraic, Assignment, Rate};
use crate::core::{AbstractRule, Model, SBase, UnitDefinition};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlElement, XmlProperty, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::{HashMap, HashSet};

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

impl CanTypeCheck for Model {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if let Some(list_of_function_definition) = self.function_definitions().get() {
            type_check_of_list(&list_of_function_definition, issues);
        }
        if let Some(list_of_unit_definitions) = self.unit_definitions().get() {
            type_check_of_list(&list_of_unit_definitions, issues);
        }
        if let Some(list_of_compartments) = self.compartments().get() {
            type_check_of_list(&list_of_compartments, issues);
        }
        if let Some(list_of_species) = self.species().get() {
            type_check_of_list(&list_of_species, issues);
        }
        if let Some(list_of_parameters) = self.parameters().get() {
            type_check_of_list(&list_of_parameters, issues);
        }
        if let Some(list_of_initial_assignment) = self.initial_assignments().get() {
            type_check_of_list(&list_of_initial_assignment, issues);
        }
        if let Some(list_of_rules) = self.rules().get() {
            type_check_of_list(&list_of_rules, issues);
        }
        if let Some(list_of_constraint) = self.constraints().get() {
            type_check_of_list(&list_of_constraint, issues);
        }
        if let Some(list_of_reactions) = self.reactions().get() {
            type_check_of_list(&list_of_reactions, issues);
        }
        if let Some(list_of_events) = self.events().get() {
            type_check_of_list(&list_of_events, issues);
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

    pub(crate) fn apply_rule_10601(&self, xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
        let mut bipartite_graph: HashMap<Vertex, Vec<Vertex>> = HashMap::new();

        self.load_vertices(&mut bipartite_graph);
        compute_edges(&mut bipartite_graph);
    }

    fn load_vertices(&self, graph: &mut HashMap<Vertex, Vec<Vertex>>) {
        let mut vertices_equation: Vec<(VertexKey, XmlElement)> = Vec::new();
        let mut vertices_variable: Vec<(VertexKey, XmlElement)> = Vec::new();

        self.get_vertices_species(&mut vertices_equation, &mut vertices_variable);
        self.get_vertices_rules(&mut vertices_equation);
        self.get_vertices_reactions(&mut vertices_equation, &mut vertices_variable);
        self.get_vertices_compartments(&mut vertices_variable);
        self.get_vertices_parameters(&mut vertices_variable);

        insert_vertices(graph, vertices_equation, Equation);
        insert_vertices(graph, vertices_variable, Variable);
    }

    /// Performs the following:
    /// - get equation vertices as of kinetic law
    /// - also get variable vertices as of SpeciesReference objects (reactants and products of a Reaction)
    /// - also get variable vertices as of Reaction objects
    fn get_vertices_reactions(
        &self,
        vertices_equation: &mut Vec<(VertexKey, XmlElement)>,
        vertices_variable: &mut Vec<(VertexKey, XmlElement)>,
    ) {
        self.reactions().get().and_then(|reactions| {
            Some(reactions.iter().for_each(|reaction| {
                reaction.kinetic_law().get().and_then(|kl| {
                    Some(vertices_equation.push((VertexKey::KineticLaw, kl.xml_element().clone())))
                });
                reaction.reactants().get().and_then(|reactants| {
                    Some(
                        reactants
                            .iter()
                            .filter(|reactant| !reactant.constant().get())
                            .for_each(|r| {
                                vertices_variable
                                    .push((VertexKey::SpeciesReference, r.xml_element().clone()))
                            }),
                    )
                });
                reaction.products().get().and_then(|products| {
                    Some(
                        products
                            .iter()
                            .filter(|product| !product.constant().get())
                            .for_each(|p| {
                                vertices_variable
                                    .push((VertexKey::SpeciesReference, p.xml_element().clone()))
                            }),
                    )
                });
                vertices_variable.push((VertexKey::Reaction, reaction.xml_element().clone()))
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
        vertices_equation: &mut Vec<(VertexKey, XmlElement)>,
        vertices_variable: &mut Vec<(VertexKey, XmlElement)>,
    ) {
        self.species().get().and_then(|species| {
            Some(species.iter().for_each(|s| {
                if !s.boundary_condition().get()
                    && !s.constant().get()
                    && s.is_referenced_by_reaction(&self)
                {
                    vertices_equation.push((VertexKey::Species, s.xml_element().clone()))
                }
                if !s.constant().get() {
                    vertices_variable.push((VertexKey::Species, s.xml_element().clone()))
                }
            }))
        });
    }

    /// Performs the following:
    /// - get equation vertices as of rules
    fn get_vertices_rules(&self, vertices_equation: &mut Vec<(VertexKey, XmlElement)>) {
        self.rules().get().and_then(|rules| {
            Some(rules.iter().for_each(|r| match r.cast() {
                Assignment(r) => {
                    vertices_equation.push((VertexKey::AssignmentRule, r.xml_element().clone()))
                }
                Rate(r) => vertices_equation.push((VertexKey::RateRule, r.xml_element().clone())),
                Algebraic(r) => {
                    vertices_equation.push((VertexKey::AlgebraicRule, r.xml_element().clone()))
                }
                _ => (),
            }))
        });
    }

    /// Performs the following:
    /// - get variable vertices as of compartments having constant=false
    fn get_vertices_compartments(
        &self,
        vertices_variable: &mut Vec<(VertexKey, XmlElement)>,
    ) -> Option<()> {
        let compartments = self.compartments().get()?;
        compartments
            .iter()
            .filter(|c| !c.constant().get())
            .for_each(|c| {
                vertices_variable.push((VertexKey::Compartment, c.xml_element().clone()))
            });
        Some(())
    }

    /// Performs the following:
    /// - get variable vertices as of parameters having constant=false
    fn get_vertices_parameters(&self, vertices_variable: &mut Vec<(VertexKey, XmlElement)>) {
        self.parameters().get().and_then(|parameters| {
            Some(
                parameters
                    .iter()
                    .filter(|p| !p.constant().get())
                    .for_each(|p| {
                        vertices_variable.push((VertexKey::Parameter, p.xml_element().clone()))
                    }),
            )
        });
    }
}

#[derive(PartialEq, Eq, Hash)]
enum VertexKey {
    Species,
    AssignmentRule,
    RateRule,
    AlgebraicRule,
    KineticLaw,
    Compartment,
    Parameter,
    SpeciesReference,
    Reaction,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum VertexType {
    Equation,
    Variable,
}

#[derive(Hash, PartialEq, Eq)]
struct Vertex {
    v_key: VertexKey,
    v_type: VertexType,
    xml_element: XmlElement,
}

fn insert_vertices(
    graph: &mut HashMap<Vertex, Vec<Vertex>>,
    key_element_list: Vec<(VertexKey, XmlElement)>,
    vertex_type: VertexType,
) {
    for key_element in key_element_list {
        graph.insert(
            Vertex {
                v_key: key_element.0,
                v_type: vertex_type,
                xml_element: key_element.1,
            },
            Vec::new(),
        );
    }
}

fn compute_edges(graph: &mut HashMap<Vertex, Vec<Vertex>>) {}
