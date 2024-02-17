use crate::core::validation::{apply_rule_10102, get_allowed_children};
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
        if self.reactions().is_set() {
            self.validate_list_of_reactions(issues);
        }
        if self.events().is_set() {
            self.validate_list_of_events(issues);
        }
    }

    fn validate_list_of_function_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.function_definitions().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let function_def = list.get(i);
            if allowed.contains(&function_def.tag_name().as_str()) {
                function_def.validate(issues);
            }
        }
    }

    fn validate_list_of_unit_definitions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.unit_definitions().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let unit_def = list.get(i);
            if allowed.contains(&unit_def.tag_name().as_str()) {
                unit_def.validate(issues);
            }
        }
    }

    fn validate_list_of_compartments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.compartments().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let compartment = list.get(i);
            if allowed.contains(&compartment.tag_name().as_str()) {
                compartment.validate(issues);
            }
        }
    }

    fn validate_list_of_species(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.species().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let species = list.get(i);
            if allowed.contains(&species.tag_name().as_str()) {
                species.validate(issues);
            }
        }
    }

    fn validate_list_of_parameters(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.parameters().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let parameter = list.get(i);
            if allowed.contains(&parameter.tag_name().as_str()) {
                parameter.validate(issues);
            }
        }
    }

    fn validate_list_of_initial_assignments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.initial_assignments().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let initial_assignment = list.get(i);
            if allowed.contains(&initial_assignment.tag_name().as_str()) {
                initial_assignment.validate(issues);
            }
        }
    }

    fn validate_list_of_rules(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.rules().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let rule = list.get(i);
            if allowed.contains(&rule.tag_name().as_str()) {
                rule.validate(issues);
            }
        }
    }

    fn validate_list_of_constraints(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.constraints().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let constraint = list.get(i);
            if allowed.contains(&constraint.tag_name().as_str()) {
                constraint.validate(issues);
            }
        }
    }

    fn validate_list_of_reactions(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.reactions().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let reaction = list.get(i);
            if allowed.contains(&reaction.tag_name().as_str()) {
                reaction.validate(issues);
            }
        }
    }

    fn validate_list_of_events(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.events().get().unwrap();
        let allowed = get_allowed_children(list.xml_element());
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let event = list.get(i);
            if allowed.contains(&event.tag_name().as_str()) {
                event.validate(issues);
            }
        }
    }

    // fn apply_rule_10301(&self, issues: &mut Vec<SbmlIssue>) {}
}
