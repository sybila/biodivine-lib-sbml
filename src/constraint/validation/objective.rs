use crate::constraint::Objective;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::RequiredXmlChild;
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Objective {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        if let flux_objectives = self.flux_objectives().get() {
            validate_list_of_objects(&flux_objectives, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for Objective {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        type_check_of_list(&self.flux_objectives().get(), issues)
    }
}
