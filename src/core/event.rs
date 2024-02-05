use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{
    OptionalChild, RequiredProperty, RequiredXmlProperty, XmlDefault, XmlDocument, XmlElement,
    XmlList,
};
use macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Event(XmlElement);

impl XmlDefault for Event {
    fn default(document: XmlDocument) -> Self {
        Event::new(document, false)
    }
}

impl Event {
    pub fn new(document: XmlDocument, use_values_from_trigger_time: bool) -> Self {
        let obj = Event::new_empty(document, "event");
        obj.use_values_from_trigger_time()
            .set(&use_values_from_trigger_time);
        obj
    }

    pub fn use_values_from_trigger_time(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("useValuesFromTriggerTime")
    }

    pub fn trigger(&self) -> OptionalChild<Trigger> {
        self.optional_sbml_child("trigger")
    }

    pub fn priority(&self) -> OptionalChild<Priority> {
        self.optional_sbml_child("priority")
    }

    pub fn delay(&self) -> OptionalChild<Delay> {
        self.optional_sbml_child("delay")
    }

    pub fn event_assignments(&self) -> OptionalChild<XmlList<EventAssignment>> {
        self.optional_sbml_child("listOfEventAssignments")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Trigger(XmlElement);

impl XmlDefault for Trigger {
    fn default(document: XmlDocument) -> Self {
        Trigger::new(document, false, false)
    }
}

impl Trigger {
    pub fn new(document: XmlDocument, persistent: bool, initial_value: bool) -> Self {
        let obj = Trigger::new_empty(document, "trigger");
        obj.persistent().set(&persistent);
        obj.initial_value().set(&initial_value);
        obj
    }

    pub fn initial_value(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("initialValue")
    }

    pub fn persistent(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("persistent")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Priority(XmlElement);

impl XmlDefault for Priority {
    fn default(document: XmlDocument) -> Self {
        Priority::new_empty(document, "priority")
    }
}

impl Priority {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Delay(XmlElement);

impl XmlDefault for Delay {
    fn default(document: XmlDocument) -> Self {
        Delay::new_empty(document, "delay")
    }
}

impl Delay {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct EventAssignment(XmlElement);

impl EventAssignment {
    pub fn new(document: XmlDocument, variable: &String) -> Self {
        let obj = EventAssignment::new_empty(document, "eventAssignment");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredProperty<String> {
        self.required_sbml_property("variable")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
