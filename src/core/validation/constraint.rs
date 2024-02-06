use crate::core::validation::apply_rule_10102;
use crate::core::Constraint;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Constraint {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}
