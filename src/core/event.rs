use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::math::Math;
use crate::xml::{
    Document, OptionalChild, OptionalSbmlChild, RequiredSbmlProperty, RequiredXmlProperty,
    XmlDefault, XmlElement, XmlList,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "event", namespace = NS_SBML_CORE)]
pub struct Event(XmlElement);

impl XmlDefault for Event {
    fn default(document: Document) -> Self {
        Event::new(document, false)
    }
}

impl Event {
    pub fn new(document: Document, use_values_from_trigger_time: bool) -> Self {
        let obj = Event::new_empty(document);
        obj.use_values_from_trigger_time()
            .set(&use_values_from_trigger_time);
        obj
    }

    pub fn use_values_from_trigger_time(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("useValuesFromTriggerTime")
    }

    pub fn trigger(&self) -> OptionalSbmlChild<Trigger> {
        self.optional_sbml_child("trigger")
    }

    pub fn priority(&self) -> OptionalSbmlChild<Priority> {
        self.optional_sbml_child("priority")
    }

    pub fn delay(&self) -> OptionalSbmlChild<Delay> {
        self.optional_sbml_child("delay")
    }

    pub fn event_assignments(&self) -> OptionalSbmlChild<XmlList<EventAssignment>> {
        self.optional_sbml_child("listOfEventAssignments")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "trigger", namespace = NS_SBML_CORE)]
pub struct Trigger(XmlElement);

impl XmlDefault for Trigger {
    fn default(document: Document) -> Self {
        Trigger::new(document, false, false)
    }
}

impl Trigger {
    pub fn new(document: Document, persistent: bool, initial_value: bool) -> Self {
        let obj = Trigger::new_empty(document);
        obj.persistent().set(&persistent);
        obj.initial_value().set(&initial_value);
        obj
    }

    pub fn initial_value(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("initialValue")
    }

    pub fn persistent(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("persistent")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "priority", namespace = NS_SBML_CORE)]
pub struct Priority(XmlElement);

impl XmlDefault for Priority {
    fn default(document: Document) -> Self {
        Priority::new_empty(document)
    }
}

impl Priority {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "delay", namespace = NS_SBML_CORE)]
pub struct Delay(XmlElement);

impl XmlDefault for Delay {
    fn default(document: Document) -> Self {
        Delay::new_empty(document)
    }
}

impl Delay {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "eventAssignment", namespace = NS_SBML_CORE)]
pub struct EventAssignment(XmlElement);

impl EventAssignment {
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = EventAssignment::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("variable")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
