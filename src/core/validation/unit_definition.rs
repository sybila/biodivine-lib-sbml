use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, validate_list_of_objects, SbmlValidable,
};
use crate::core::{SBase, UnitDefinition};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for UnitDefinition {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(list_of_units) = self.units().get() {
            validate_list_of_objects(&list_of_units, issues, identifiers);
        }
    }
}
