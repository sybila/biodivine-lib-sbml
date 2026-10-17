use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{Delay, Event, EventAssignment, MetaId, Model, Priority, SId, Trigger};
use crate::xml::{OptionalXmlChild, XmlList, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Event {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(trigger) = self.trigger().get() {
            trigger.validate(issues, identifiers, meta_ids, check)?;
        }
        if let Some(priority) = self.priority().get() {
            priority.validate(issues, identifiers, meta_ids, check)?;
        }
        if let Some(delay) = self.delay().get() {
            delay.validate(issues, identifiers, meta_ids, check)?;
        }
        if let Some(list_of_event_assignments) = self.event_assignments().get() {
            validate_list_of_objects(
                &list_of_event_assignments,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
            Event::apply_rule_10305(&list_of_event_assignments, issues);
            Event::apply_rule_10306(&list_of_event_assignments, issues);
        }
        Ok(())
    }
}

impl CanTypeCheck for Event {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));

        if let Some(trigger) = self.trigger().get() {
            trigger.type_check(issues, check)?;
        }
        if let Some(priority) = self.priority().get() {
            priority.type_check(issues, check)?;
        }
        if let Some(delay) = self.delay().get() {
            delay.type_check(issues, check)?;
        }
        if let Some(list_of_event_assignments) = self.event_assignments().get() {
            type_check_of_list(&list_of_event_assignments, issues, check)?;
        }
        Ok(())
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
        let mut variables: HashSet<SId> = HashSet::new();

        for event_assignment in list_of_event_assignments.iter() {
            // A missing or unparsable `variable` is reported by the type check of the element.
            let Some(variable) = XmlProperty::get_checked(&event_assignment.variable())
                .ok()
                .flatten()
            else {
                continue;
            };
            if variables.contains(&variable) {
                let message = format!(
                    "The variable ('{variable}') of <eventAssignment> is \
                already present in the <listOfEventAssignments>."
                );
                issues.push(SbmlIssue::new("10305", &event_assignment, message));
            } else {
                variables.insert(variable);
            }
        }
    }

    /// ### Rule 10306
    /// An identifier used as the value of the attribute variable of an [EventAssignment] object
    /// cannot also appear as the value of the variable attribute in an
    /// [AssignmentRule](crate::core::rule::AssignmentRule) object. In other words, a given model
    /// component cannot be the subject of both an assignment rule and an assignment by an event.
    pub(crate) fn apply_rule_10306(
        list_of_event_assignments: &XmlList<EventAssignment>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let model = Model::for_child_element(list_of_event_assignments.xml_element()).unwrap();
        let assignment_rule_variables = model.assignment_rule_variables();

        for event_assignment in list_of_event_assignments.iter() {
            let Some(value) = XmlProperty::get_checked(&event_assignment.variable())
                .ok()
                .flatten()
            else {
                continue;
            };
            if assignment_rule_variables.contains(&value) {
                let message = format!(
                    "The variable ('{value}') of <eventAssignment> found \
                as a variable of <assignmentRule>."
                );
                issues.push(SbmlIssue::new("10306", &event_assignment, message));
            }
        }
    }
}

impl SbmlValidable for Trigger {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Trigger {}

impl SbmlValidable for Priority {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Priority {}

impl SbmlValidable for Delay {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Delay {}

impl SbmlValidable for EventAssignment {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for EventAssignment {}
