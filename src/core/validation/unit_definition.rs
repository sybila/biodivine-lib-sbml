use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10307, apply_rule_10308, apply_rule_10309, apply_rule_10310, apply_rule_10311,
    apply_rule_10312, apply_rule_10401, apply_rule_10402, validate_list_of_objects, SbmlValidable,
};
use crate::core::{SBase, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlList, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10311("id", id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
            apply_rule_10402(&annotation, issues);
        }
        if let Some(list_of_units) = self.units().get() {
            validate_list_of_objects(&list_of_units, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for UnitDefinition {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if let Some(list_of_units) = self.units().get() {
            type_check_of_list(&list_of_units, issues);
        }
    }
}

impl UnitDefinition {
    pub(crate) fn apply_rule_10302(
        list_of_unit_definitions: &XmlList<UnitDefinition>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let mut identifiers: HashSet<String> = HashSet::new();

        for unit_definition in list_of_unit_definitions.iter() {
            let Some(id) = unit_definition.id().get() else {
                continue;
            };

            if identifiers.contains(&id) {
                let message = format!("The identifier ('{id}') of <unitDefinition> is already present in the <listOfUnitDefinitions>.");
                issues.push(SbmlIssue::new_error("10302", &unit_definition, message));
            } else {
                identifiers.insert(id);
            }
        }
    }
}
