use crate::cancellation::CancellationCheck;
use crate::constraint::Objective;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::{RequiredXmlChild, XmlChild, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Objective {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        validate_list_of_objects(
            &self.flux_objectives().get(),
            issues,
            identifiers,
            meta_ids,
            check,
        )?;
        Ok(())
    }
}

impl CanTypeCheck for Objective {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.flux_objectives().get_raw().is_some() {
            // We can only check the `objectives` if they exist. They might not, even though
            // they are required... (no need to do this after type check)
            type_check_of_list(&self.flux_objectives().get(), issues, check)?
        }
        Ok(())
    }
}
