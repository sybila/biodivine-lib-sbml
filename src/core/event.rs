use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::math::{Math, MathNode};
use crate::xml::{
    Document, OptionalChild, OptionalSbmlChild, OptionalXmlChild, RequiredSbmlProperty,
    RequiredXmlProperty, XmlDefault, XmlElement, XmlList,
};
use embed_doc_image::embed_doc_image;
use sbml_macros::{SBase, XmlWrapper};

/// The `<event>` element of the SBML core specification.
///
/// ![UML][sbml-event]
#[embed_doc_image("sbml-event", "docs-images/uml-event.svg")]
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "event", namespace = NS_SBML_CORE)]
pub struct Event(XmlElement);

impl XmlDefault for Event {
    fn default(document: Document) -> Self {
        Event::new(document, false)
    }
}

impl Event {
    /// Creates a new `<event>` element with the given `use_values_from_trigger_time`.
    pub fn new(document: Document, use_values_from_trigger_time: bool) -> Self {
        let obj = Event::new_empty(document);
        obj.use_values_from_trigger_time()
            .set(&use_values_from_trigger_time);
        obj
    }

    /// The `useValuesFromTriggerTime` attribute of this `<event>` element.
    pub fn use_values_from_trigger_time(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("useValuesFromTriggerTime")
    }

    /// The `trigger` child of this `<event>` element.
    pub fn trigger(&self) -> OptionalSbmlChild<'_, Trigger> {
        self.optional_sbml_child("trigger")
    }

    /// The `priority` child of this `<event>` element.
    pub fn priority(&self) -> OptionalSbmlChild<'_, Priority> {
        self.optional_sbml_child("priority")
    }

    /// The `delay` child of this `<event>` element.
    pub fn delay(&self) -> OptionalSbmlChild<'_, Delay> {
        self.optional_sbml_child("delay")
    }

    /// The `listOfEventAssignments` child of this `<event>` element, as a list of [`EventAssignment`].
    pub fn event_assignments(&self) -> OptionalSbmlChild<'_, XmlList<EventAssignment>> {
        self.optional_sbml_child("listOfEventAssignments")
    }
}

/// The `<trigger>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "trigger", namespace = NS_SBML_CORE)]
pub struct Trigger(XmlElement);

impl XmlDefault for Trigger {
    fn default(document: Document) -> Self {
        Trigger::new(document, false, false)
    }
}

impl Trigger {
    /// Creates a new `<trigger>` element with the given `persistent`, `initial_value`.
    pub fn new(document: Document, persistent: bool, initial_value: bool) -> Self {
        let obj = Trigger::new_empty(document);
        obj.persistent().set(&persistent);
        obj.initial_value().set(&initial_value);
        obj
    }

    /// The `initialValue` attribute of this `<trigger>` element.
    pub fn initial_value(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("initialValue")
    }

    /// The `persistent` attribute of this `<trigger>` element.
    pub fn persistent(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("persistent")
    }

    /// The `math` child of this `<trigger>` element.
    pub fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}

/// The `<priority>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "priority", namespace = NS_SBML_CORE)]
pub struct Priority(XmlElement);

impl XmlDefault for Priority {
    fn default(document: Document) -> Self {
        Priority::new_empty(document)
    }
}

impl Priority {
    /// The `math` child of this `<priority>` element.
    pub fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}

/// The `<delay>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "delay", namespace = NS_SBML_CORE)]
pub struct Delay(XmlElement);

impl XmlDefault for Delay {
    fn default(document: Document) -> Self {
        Delay::new_empty(document)
    }
}

impl Delay {
    /// The `math` child of this `<delay>` element.
    pub fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}

/// The `<eventAssignment>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "eventAssignment", namespace = NS_SBML_CORE)]
pub struct EventAssignment(XmlElement);

impl EventAssignment {
    /// Creates a new `<eventAssignment>` element with the given `variable`.
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = EventAssignment::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    /// The `variable` attribute of this `<eventAssignment>` element.
    pub fn variable(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("variable")
    }

    /// The `math` child of this `<eventAssignment>` element.
    pub fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}
