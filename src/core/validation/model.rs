use crate::core::validation::{apply_rule_10102, apply_rule_10301, validate_list_of_objects};
use crate::core::{AbstractRule, Model, SBase, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl Model {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(list_of_function_definition) = self.function_definitions().get() {
            validate_list_of_objects(&list_of_function_definition, issues, identifiers);
        }
        if let Some(list_of_unit_definitions) = self.unit_definitions().get() {
            validate_list_of_objects(&list_of_unit_definitions, issues, identifiers);
            UnitDefinition::apply_rule_10302(&list_of_unit_definitions, issues);
        }
        if let Some(list_of_compartments) = self.compartments().get() {
            validate_list_of_objects(&list_of_compartments, issues, identifiers);
        }
        if let Some(list_of_species) = self.species().get() {
            validate_list_of_objects(&list_of_species, issues, identifiers);
        }
        if let Some(list_of_parameters) = self.parameters().get() {
            validate_list_of_objects(&list_of_parameters, issues, identifiers);
        }
        if let Some(list_of_initial_assignment) = self.initial_assignments().get() {
            validate_list_of_objects(&list_of_initial_assignment, issues, identifiers);
        }
        if let Some(list_of_rules) = self.rules().get() {
            validate_list_of_objects(&list_of_rules, issues, identifiers);
            AbstractRule::apply_rule_10304(&list_of_rules, issues);
        }
        if let Some(list_of_constraint) = self.constraints().get() {
            validate_list_of_objects(&list_of_constraint, issues, identifiers);
        }
        if let Some(list_of_reactions) = self.reactions().get() {
            validate_list_of_objects(&list_of_reactions, issues, identifiers);
        }
        if let Some(list_of_events) = self.events().get() {
            validate_list_of_objects(&list_of_events, issues, identifiers);
        }
    }
}
