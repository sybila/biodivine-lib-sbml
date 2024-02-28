use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309,
    apply_rule_10310, apply_rule_10311, SanityCheckable, SbmlValidable,
};
use crate::core::{Compartment, SBase};
use crate::xml::{OptionalXmlProperty, RequiredXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Compartment {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let meta_id = self.meta_id();
        let id = self.id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(Some(id.get()), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(Some(id.get()), xml_element, issues);
        apply_rule_10311("units", self.units().get(), xml_element, issues);
    }
}

impl SanityCheckable for Compartment {}
