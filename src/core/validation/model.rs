use crate::core::Model;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Model {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        self.apply_rule_10102(issues);

        if self.function_definitions().is_set() {
            self.validate_list_of_function_definitions(issues);
        }
    }

    fn validate_list_of_function_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.function_definitions().get().unwrap();
        list.apply_rule_10102(issues);

        for i in 0..list.len() {
            let function_def = list.get(i);
            function_def.validate(issues);
        }
    }
}
