use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{apply_rule_10311, validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SBase, SId, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlList, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let id = self.id();

        apply_rule_10311("id", id.get_raw(), xml_element, issues);

        if let Some(list_of_units) = self.units().get() {
            validate_list_of_objects(&list_of_units, issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for UnitDefinition {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));

        if let Some(list_of_units) = self.units().get() {
            type_check_of_list(&list_of_units, issues, check)?;
        }
        Ok(())
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
                issues.push(SbmlIssue::new("10302", &unit_definition, message));
            } else {
                identifiers.insert(id);
            }
        }
    }
}
