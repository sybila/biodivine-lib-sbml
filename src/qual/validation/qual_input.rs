use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::qual::qual_input::QualInput;
use crate::qual::{QualitativeSpecies, TransitionInputEffect};
use crate::xml::{RequiredXmlProperty, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for QualInput {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        apply_rule_qual_20509_and_20508(self, issues)
    }
}

impl CanTypeCheck for QualInput {}

fn apply_rule_qual_20509_and_20508(element: &QualInput, issues: &mut Vec<SbmlIssue>) {
    let qual_species =
        element.find_by_sid::<QualitativeSpecies>(&element.qualitative_species().get());

    if qual_species.is_none() || qual_species.clone().unwrap().tag_name() != "qualitativeSpecies" {
        let message = "Attribute [qualitativeSpecies] does not refer to an existing QualitativeSpecies element!";
        issues.push(SbmlIssue::new_error("qual-20508", element, message));
        return;
    }

    if element.transition_effect().get_raw().is_some()
        && qual_species.unwrap().constant().get()
        && element.transition_effect().get() == TransitionInputEffect::Consumption
    {
        let message = "When qualitativeSpecies attribute const is set to true the transitionEffect attribute can not be set to consumption!".to_string();
        issues.push(SbmlIssue::new_error("qual-20509", element, message));
    }
}
