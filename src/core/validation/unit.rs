use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309,
    apply_rule_10310, apply_rule_10312, apply_rule_10401, apply_rule_10402, apply_rule_10404,
    SanityCheckable, SbmlValidable,
};
use crate::core::{SBase, Unit};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Unit {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
            apply_rule_10402(&annotation, issues);
            apply_rule_10404(xml_element, issues);
        }
    }
}

impl SanityCheckable for Unit {}
