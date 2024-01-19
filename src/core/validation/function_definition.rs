use crate::core::validation::apply_rule_10102;
use crate::core::FunctionDefinition;
use crate::xml::XmlWrapper;
use crate::SbmlIssue;

impl FunctionDefinition {
    pub(crate) fn validate(&self, _issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), _issues);
    }
}
