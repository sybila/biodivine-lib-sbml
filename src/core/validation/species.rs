use crate::core::validation::apply_rule_10102;
use crate::core::Species;
use crate::xml::XmlWrapper;
use crate::SbmlIssue;

impl Species {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);
    }
}
