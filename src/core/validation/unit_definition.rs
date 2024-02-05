use crate::core::validation::{apply_rule_10102, get_allowed_children};
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

        let allowed = get_allowed_children(list.xml_element());
        for i in 0..list.len() {
            let unit = list.get(i);
            if allowed.contains(&unit.tag_name().as_str()) {
                unit.validate(issues);
            }
        }
    }
}
