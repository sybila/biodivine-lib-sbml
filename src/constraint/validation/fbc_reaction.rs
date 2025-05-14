use crate::constraint::fbc_reaction::FbcReaction;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, Parameter, SId};
use crate::xml::{OptionalXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for FbcReaction {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(lowerBound) = self.lower_flux_bound().get() {
            apply_rule_fbc_20705(self, issues, lowerBound);
        }

        if let Some(upperBound) = self.upper_flux_bound().get() {
            apply_rule_fbc_20705(self, issues, upperBound);
        }
    }
}

impl CanTypeCheck for FbcReaction {}

pub fn apply_rule_fbc_20705(element: &FbcReaction, issues: &mut Vec<SbmlIssue>, sid_ref: SId) {
    let found = element.find_by_sid::<Parameter>(&sid_ref);

    if found.is_none() {
        let message =
            "Attribute of Flux Bound does not refer to the existing Parameter element!".to_string();
        issues.push(SbmlIssue::new_error(
            "fbc:20707",
            element.xml_element(),
            message,
        ))
    }
}
