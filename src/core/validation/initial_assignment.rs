use crate::core::validation::apply_rule_10102;
use crate::core::InitialAssignment;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl InitialAssignment {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}
