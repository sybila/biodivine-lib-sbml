use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, sanity_check, sanity_check_of_list,
    validate_list_of_objects, SanityCheckable, SbmlValidable,
};
use crate::core::{Delay, Event, EventAssignment, Priority, SBase, Trigger};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlList, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Event {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(trigger) = self.trigger().get() {
            trigger.validate(issues, identifiers);
        }
        if let Some(priority) = self.priority().get() {
            priority.validate(issues, identifiers);
        }
        if let Some(delay) = self.delay().get() {
            delay.validate(issues, identifiers);
        }
        if let Some(list_of_event_assignments) = self.event_assignments().get() {
            validate_list_of_objects(&list_of_event_assignments, issues, identifiers);
            Event::apply_rule_10305(&list_of_event_assignments, issues);
        }
    }
}

impl SanityCheckable for Event {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);

        if let Some(trigger) = self.trigger().get() {
            trigger.sanity_check(issues);
        }
        if let Some(priority) = self.priority().get() {
            priority.sanity_check(issues);
        }
        if let Some(delay) = self.delay().get() {
            delay.sanity_check(issues);
        }
        if let Some(list_of_event_assignments) = self.event_assignments().get() {
            sanity_check_of_list(&list_of_event_assignments, issues);
        }
    }
}

impl Event {
    /// ### Rule 10305
    /// In every [Event] object, the value of the attribute *variable* within each [EventAssignment]
    /// subobject must be unique across the set of all such [EventAssignment] subobjects within
    /// that particular [Event] object. In other words, a single [Event] cannot make more than one
    /// assignment to the same model component.
    pub(crate) fn apply_rule_10305(
        list_of_event_assignments: &XmlList<EventAssignment>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let mut variables: HashSet<String> = HashSet::new();

        for event_assignment in list_of_event_assignments.iter() {
            let variable = event_assignment.variable().get();
            if variables.contains(&variable) {
                let message = format!(
                    "The variable ('{variable}') of <eventAssignment> is \
                already present in the <listOfEventAssignments>."
                );
                issues.push(SbmlIssue::new_error("10305", &event_assignment, message));
            } else {
                variables.insert(variable);
            }
        }
    }
}

impl SbmlValidable for Trigger {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for Trigger {}

impl SbmlValidable for Priority {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for Priority {}

impl SbmlValidable for Delay {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for Delay {}

impl SbmlValidable for EventAssignment {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for EventAssignment {}
