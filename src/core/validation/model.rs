use crate::core::Model;
use crate::xml::XmlWrapper;
use crate::SbmlIssue;
use std::ops::Deref;

impl Model {
    pub(crate) fn validate(&self, _issues: &mut Vec<SbmlIssue>) {
        unimplemented!()
    }

    pub(crate) fn apply_rule_10102(&self, _issues: &mut Vec<SbmlIssue>) {
        let _rule_number = "10102".to_string();
        let _doc = self.document().read().unwrap().deref();

        todo!()
    }
}
