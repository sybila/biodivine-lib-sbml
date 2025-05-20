use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
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

        if let Some(listOfInputs) = self.inputs().get() {
            validate_list_of_objects(&listOfInputs, issues, identifiers, meta_ids);
        }

        if let Some(listOfOutputs) = self.outputs().get() {
            validate_list_of_objects(&listOfOutputs, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for Transition {}
