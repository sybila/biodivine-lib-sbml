use crate::constraint::GeneProduct;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId, Species};
use crate::xml::OptionalXmlProperty;
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for GeneProduct {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(associated_species) = self.associated_species().get() {
            apply_rule_fbc_21207(self, issues, associated_species)
        }
    }
}

impl CanTypeCheck for GeneProduct {}

pub fn apply_rule_fbc_21207(element: &GeneProduct, issues: &mut Vec<SbmlIssue>, sid_ref: SId) {
    let found = element.find_by_sid::<Species>(&sid_ref);

    if found.is_none() {
        let message =
            "Attribute [associatedSpecies] does not refer to an existing Species element!"
                .to_string();
        issues.push(SbmlIssue::new_error("fbc:20908", element, message))
    }
}
