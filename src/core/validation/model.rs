use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10311, apply_rule_10313, validate_list_of_objects, SbmlValidable,
};
use crate::core::{AbstractRule, MetaId, Model, SId, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlElement, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Model {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();

        self.apply_rule_10311(xml_element, issues);
        self.apply_rule_10313(xml_element, issues);

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

        apply_rule_10311(
            sbstnc_units.simple_name(),
            sbstnc_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10311(
            volume_units.simple_name(),
            volume_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10311(
            area_units.simple_name(),
            area_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10311(
            length_units.simple_name(),
            length_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10311(
            time_units.simple_name(),
            time_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10311(
            extent_units.simple_name(),
            extent_units.get_raw(),
            xml_element,
            issues,
        );
    }
    pub(crate) fn apply_rule_10313(&self, xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
        let sbstnc_units = self.substance_units();
        let volume_units = self.volume_units();
        let area_units = self.area_units();
        let length_units = self.length_units();
        let time_units = self.time_units();
        let extent_units = self.extent_units();

        apply_rule_10313(
            sbstnc_units.simple_name(),
            sbstnc_units.get(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            volume_units.simple_name(),
            volume_units.get(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            area_units.simple_name(),
            area_units.get(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            length_units.simple_name(),
            length_units.get(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            time_units.simple_name(),
            time_units.get(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            extent_units.simple_name(),
            extent_units.get(),
            xml_element,
            issues,
        );
    }
}
