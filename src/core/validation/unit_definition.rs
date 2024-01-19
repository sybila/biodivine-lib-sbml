use crate::core::validation::apply_rule_10102;
use crate::core::UnitDefinition;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl UnitDefinition {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if self.units().is_set() {
            self.validate_list_of_units(issues);
        }
    }

    fn validate_list_of_units(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.units().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let unit = list.get(i);
            // TODO: might panic if some child of the list is not allowed by SBML rules.
            // SOLUTION: check if tag name is in keys of ALLOWED_CHILDREN
            unit.validate(issues);
        }
    }
}
