use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::qual::Transition;
use crate::xml::{OptionalXmlChild, RequiredXmlChild};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Transition {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        validate_list_of_objects(&self.function_terms().get(), issues, identifiers, meta_ids);

        if let Some(list_of_inputs) = self.inputs().get() {
            validate_list_of_objects(&list_of_inputs, issues, identifiers, meta_ids);
        }

        if let Some(list_of_outputs) = self.outputs().get() {
            validate_list_of_objects(&list_of_outputs, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for Transition {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        type_check_of_list(&self.function_terms().get(), issues);

        if let Some(list_of_inputs) = self.inputs().get() {
            type_check_of_list(&list_of_inputs, issues);
        }

        if let Some(list_of_outputs) = self.outputs().get() {
            type_check_of_list(&list_of_outputs, issues);
        }
    }
}
