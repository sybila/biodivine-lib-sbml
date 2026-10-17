use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::qual::Transition;
use crate::xml::{OptionalXmlChild, RequiredXmlChild, XmlChild};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Transition {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        if self.function_terms().get_raw().is_some() {
            validate_list_of_objects(
                &self.function_terms().get(),
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }

        if let Some(list_of_inputs) = self.inputs().get() {
            validate_list_of_objects(&list_of_inputs, issues, identifiers, meta_ids, check)?;
        }

        if let Some(list_of_outputs) = self.outputs().get() {
            validate_list_of_objects(&list_of_outputs, issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Transition {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if self.function_terms().get_raw().is_some() {
            type_check_of_list(&self.function_terms().get(), issues, check)?
        }

        if let Some(list_of_inputs) = self.inputs().get() {
            type_check_of_list(&list_of_inputs, issues, check)?;
        }

        if let Some(list_of_outputs) = self.outputs().get() {
            type_check_of_list(&list_of_outputs, issues, check)?;
        }
        Ok(())
    }
}
