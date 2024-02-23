use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, sanity_check, sanity_check_of_list,
    validate_list_of_objects, SanityCheckable, SbmlValidable,
};
use crate::core::{Delay, Event, EventAssignment, Priority, SBase, Trigger};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlWrapper};
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
