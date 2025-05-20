use crate::constraint::Objective;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::{RequiredXmlChild, XmlWrapper};
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
        validate_list_of_objects(&self.flux_objectives().get(), issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for Objective {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        type_check_of_list(&self.flux_objectives().get(), issues)
    }
}
