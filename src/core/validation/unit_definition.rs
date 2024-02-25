use crate::core::validation::{
    apply_rule_10102, sanity_check, sanity_check_of_list, validate_list_of_objects,
    SanityCheckable, SbmlValidable,
};
use crate::core::{SBase, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlList, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(list_of_units) = self.units().get() {
            validate_list_of_objects(&list_of_units, issues, identifiers);
        }
    }
}

impl SanityCheckable for UnitDefinition {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);

        if let Some(list_of_units) = self.units().get() {
            sanity_check_of_list(&list_of_units, issues);
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
