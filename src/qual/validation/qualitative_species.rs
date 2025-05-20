use crate::core::sbase::SbmlUtils;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{Compartment, MetaId, SBase, SId};
use crate::qual::qual_output::QualOutput;
use crate::qual::QualitativeSpecies;
use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlElement};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for QualitativeSpecies {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
    }
}

impl CanTypeCheck for QualitativeSpecies {}

pub fn apply_rule_qual_20308(
    element: &QualitativeSpecies,
    issues: &mut Vec<SbmlIssue>,
    compartment: SId,
) {
    let comp = element.find_by_sid::<Compartment>(&compartment);

    if comp.is_none() {
        let message = "Attribute [compartment] does not refer to an existing Compartment element!";
        issues.push(SbmlIssue::new_error("qual-20308", element, message));
    }
}

pub fn apply_rule_qual_20309() {}

pub fn apply_rule_qual_20310(element: &QualitativeSpecies, issues: &mut Vec<SbmlIssue>) {}
