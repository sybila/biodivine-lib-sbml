use crate::core::validation::apply_rule_10102;
use crate::core::Model;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Model {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if self.function_definitions().is_set() {
            self.validate_list_of_function_definitions(issues);
        }
        if self.unit_definitions().is_set() {
            self.validate_list_of_unit_definitions(issues);
        }
        if self.compartments().is_set() {
            self.validate_list_of_compartments(issues);
        }
        if self.species().is_set() {
            self.validate_list_of_species(issues);
        }
    }

    fn validate_list_of_function_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.function_definitions().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let function_def = list.get(i);
            // TODO: might panic if some child of the list is not allowed by SBML rules.
            // SOLUTION: check if tag name is in keys of ALLOWED_CHILDREN
            function_def.validate(issues);
        }
    }

    fn validate_list_of_unit_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.unit_definitions().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let unit_def = list.get(i);
            // TODO: might panic if some child of the list is not allowed by SBML rules.
            // SOLUTION: check if tag name is in keys of ALLOWED_CHILDREN
            unit_def.validate(issues);
        }
    }

    fn validate_list_of_compartments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.compartments().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let compartment = list.get(i);
            // TODO: might panic if some child of the list is not allowed by SBML rules.
            // SOLUTION: check if tag name is in keys of ALLOWED_CHILDREN
            compartment.validate(issues);
        }
    }

    fn validate_list_of_species(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.species().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let species = list.get(i);
            // TODO: might panic if some child of the list is not allowed by SBML rules.
            // SOLUTION: check if tag name is in keys of ALLOWED_CHILDREN
            species.validate(issues);
        }
    }
}
