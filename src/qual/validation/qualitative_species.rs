use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{Compartment, MetaId, Model, SBase, SId};
use crate::qual::QualitativeSpecies;
use crate::xml::{OptionalXmlProperty, RequiredXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for QualitativeSpecies {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        apply_rule_qual_20308(self, issues, self.compartment().get());
        apply_rule_qual_20309(self, issues);
        apply_rule_qual_20310(self, issues);
    }
}

impl CanTypeCheck for QualitativeSpecies {}

pub fn apply_rule_qual_20308(
    element: &QualitativeSpecies,
    issues: &mut Vec<SbmlIssue>,
    compartment: SId,
) {
    let comp = element.find_by_sid::<Compartment>(&compartment);

    if comp.is_none() || comp.unwrap().tag_name() != "compartment" {
        let message = "Attribute [compartment] does not refer to an existing Compartment element!";
        issues.push(SbmlIssue::new_error("qual-20308", element, message));
    }
}

pub fn apply_rule_qual_20309(element: &QualitativeSpecies, issues: &mut Vec<SbmlIssue>) {
    let initial = element.initial_level().get();
    let max_level = element.max_level().get();

    if initial.is_some() && max_level.is_some() && initial.unwrap() > max_level.unwrap() {
        let message = "Value of attribute [initialLevel] can not be greater than value of attribute [maximumLevel]!";
        issues.push(SbmlIssue::new_error("qual-20309", element, message));
    }
}

pub fn apply_rule_qual_20310(element: &QualitativeSpecies, issues: &mut Vec<SbmlIssue>) {
    let model = Model::for_child_element(element.xml_element());
    let outputs = model.unwrap().get_all_transition_outputs();

    for output in outputs {
        if output.qualitative_species().get() == element.id().get() {
            let message = "When the [constant] element is set to true, qualitativeSpecies element can not be referenced by output!".to_string();
            issues.push(SbmlIssue::new_error(
                "qual-20310",
                element.xml_element(),
                message,
            ));
        }
    }
}
