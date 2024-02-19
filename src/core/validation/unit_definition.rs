use crate::core::validation::{apply_rule_10102, validate_list_of_objects, SbmlValidable};
use crate::core::{SBase, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlList, XmlWrapper};
use crate::{SbmlIssue, SbmlIssueSeverity};
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(list_of_units) = self.units().get() {
            validate_list_of_objects(&list_of_units, issues, identifiers);
        }
    }
}

impl UnitDefinition {
    pub(crate) fn apply_rule_10302(
        list_of_unit_definitions: &XmlList<UnitDefinition>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let mut identifiers: HashSet<String> = HashSet::new();

        for unit_definition in list_of_unit_definitions.as_vec() {
            let id = unit_definition.id().get().unwrap_or_default();
            if identifiers.contains(&id) {
                issues.push(SbmlIssue {
                    element: unit_definition.raw_element(),
                    message: format!("The identifier ('{0}') of <unitDefinition> is already present in the <listOfUnitDefinitions>.",
                                     id),
                    rule: "10302".to_string(),
                    severity: SbmlIssueSeverity::Error
                })
            } else {
                identifiers.insert(id);
            }
        }
    }
}
