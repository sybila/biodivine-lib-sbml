use crate::core::validation::{apply_rule_10102, apply_rule_10301, SanityCheckable, SbmlValidable};
use crate::core::{Constraint, SBase};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Constraint {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for Constraint {}
