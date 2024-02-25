use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, SanityCheckable, SbmlValidable,
};
use crate::core::{SBase, Species};
use crate::xml::{OptionalXmlProperty, RequiredXmlProperty, XmlWrapper};
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

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(Some(self.id().get()), xml_element, issues, identifiers);
        apply_rule_10307(self.meta_id().get(), xml_element, issues, meta_ids);
    }
}

impl SanityCheckable for Species {}
