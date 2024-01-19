use crate::core::validation::apply_rule_10102;
use crate::core::Model;
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Model {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        // TODO: might panic if some child of a list is not allowed by SBML rules.
        // SOLUTION: check if child tag name is in keys of ALLOWED_CHILDREN of parent element
        // TODO: panics if empty element looks like this: (only <tag/> passes)
        //       <tag></tag>
        //          OR
        //       <tag>
        //          "any number of /n and whitespace chars"
        //       </tag>
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
        if self.parameters().is_set() {
            self.validate_list_of_parameters(issues);
        }
        if self.initial_assignments().is_set() {
            self.validate_list_of_initial_assignments(issues);
        }
        if self.rules().is_set() {
            self.validate_list_of_rules(issues);
        }
        if self.constraints().is_set() {
            self.validate_list_of_constraints(issues);
        }
    }

    fn validate_list_of_function_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.function_definitions().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let function_def = list.get(i);
            function_def.validate(issues);
        }
    }

    fn validate_list_of_unit_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.unit_definitions().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let unit_def = list.get(i);
            unit_def.validate(issues);
        }
    }

    fn validate_list_of_compartments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.compartments().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let compartment = list.get(i);
            compartment.validate(issues);
        }
    }

    fn validate_list_of_species(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.species().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let species = list.get(i);
            species.validate(issues);
        }
    }

    fn validate_list_of_parameters(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.parameters().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let parameter = list.get(i);
            parameter.validate(issues);
        }
    }

    fn validate_list_of_initial_assignments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.initial_assignments().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let initial_assignment = list.get(i);
            initial_assignment.validate(issues);
        }
    }

    fn validate_list_of_rules(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.rules().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let rule = list.get(i);
            rule.validate(issues);
        }
    }

    fn validate_list_of_constraints(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.constraints().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let constraint = list.get(i);
            constraint.validate(issues);
        }
    }
}
