use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{apply_rule_10311, validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SBase, SId, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlList, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let id = self.id();

        apply_rule_10311("id", id.get_raw(), xml_element, issues);

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
        let mut identifiers: HashSet<SId> = HashSet::new();

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
