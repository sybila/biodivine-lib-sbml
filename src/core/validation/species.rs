use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::{
    apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309, apply_rule_10310,
    apply_rule_10311, apply_rule_10312, apply_rule_10313, apply_rule_10401, apply_rule_10402,
    SbmlValidable,
};
use crate::core::{SBase, Species};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlProperty, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Species {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();
        let sbstnc_units = self.substance_units();

        apply_rule_10301(Some(id.get()), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(Some(id.get()), xml_element, issues);
        apply_rule_10311(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);
        apply_rule_10313(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
            apply_rule_10402(&annotation, issues);
        }
    }
}

impl CanTypeCheck for Species {}
