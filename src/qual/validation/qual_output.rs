use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::qual::{QualOutput, QualitativeSpecies, TransitionOutputEffect};
use crate::xml::{RequiredXmlProperty, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for QualOutput {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        apply_rule_qual_20607_and_20608(self, issues, self.qualitative_species().get());

        if self.transition_effect().get() == TransitionOutputEffect::Production
            && !self.output_level().is_set()
        {
            let message =
                "TransitionEffect attribute is set to 'production' but the outputLevel is not set."
                    .to_string();
            issues.push(SbmlIssue::new_error("qual-20609", self, message))
        }
    }
}

impl CanTypeCheck for QualOutput {}

fn apply_rule_qual_20607_and_20608(
    element: &QualOutput,
    issues: &mut Vec<SbmlIssue>,
    qualitative_species: SId,
) {
    let qual_species = element.find_by_sid::<QualitativeSpecies>(&qualitative_species);

    if qual_species.is_none() || qual_species.clone().unwrap().tag_name() != "qualitativeSpecies" {
        let message = "Attribute [qualitativeSpecies] does not refer to an existing QualitativeSpecies element!";
        issues.push(SbmlIssue::new_error("qual-20607", element, message));
        return;
    }

    if qual_species.unwrap().constant().get() {
        let message = "The value of [constant] attribute of the [qualitativeSpecies] element has to be se to false!"
            .to_string();
        issues.push(SbmlIssue::new_error("qual-20608", element, message));
    }
}
