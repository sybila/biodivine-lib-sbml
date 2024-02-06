use crate::core::validation::apply_rule_10102;
use crate::core::{Delay, Event, EventAssignment, Priority, Trigger};
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Event {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(trigger) = self.trigger().get() {
            trigger.validate(issues);
        }
        if let Some(priority) = self.priority().get() {
            priority.validate(issues);
        }
        if let Some(delay) = self.delay().get() {
            delay.validate(issues);
        }
        if self.event_assignments().is_set() {
            self.validate_list_of_event_assignments(issues);
        }
    }

    fn validate_list_of_event_assignments(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.event_assignments().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let evt_assignment = list.get(i);
            evt_assignment.validate(issues);
        }
    }
}

impl Trigger {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl Priority {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl Delay {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl EventAssignment {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}
