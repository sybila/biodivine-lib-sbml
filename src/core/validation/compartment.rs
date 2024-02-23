use crate::core::validation::{apply_rule_10102, apply_rule_10301, SanityCheckable, SbmlValidable};
use crate::core::Compartment;
use crate::xml::{RequiredXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Compartment {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(
            Some(self.id().get()),
            self.xml_element(),
            issues,
            identifiers,
        );
    }
}

impl SanityCheckable for Compartment {}
