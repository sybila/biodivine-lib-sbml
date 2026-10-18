use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::{SBase, SId};
use crate::math::{Math, MathNode};
use crate::xml::{
    Document, OptionalChild, OptionalXmlChild, RequiredSbmlProperty, RequiredXmlProperty,
    XmlDefault, XmlElement, XmlNamedSubtype, XmlSupertype,
};
use embed_doc_image::embed_doc_image;
use sbml_macros::{SBase, XmlWrapper};

/// The kind of a rule, as [`AbstractRule::cast`] decides it from the element it wraps.
pub enum RuleTypes {
    // Other is used to represent rules that are only defined in (hypothetical) SBML extensions
    // that are not covered by this library.
    /// A rule of a kind this crate does not model (a rule only an SBML package defines).
    Other(AbstractRule),
    /// An `<algebraicRule>`: a rule with no subject identifier.
    Algebraic(AlgebraicRule),
    /// An `<assignmentRule>`: a rule that assigns the value of its formula to one identifier.
    Assignment(AssignmentRule),
    /// A `<rateRule>`: a rule that gives the time derivative of one identifier.
    Rate(RateRule),
}

/// A rule of the core specification: an object that carries a mathematical formula.
///
/// The three concrete kinds are [`AlgebraicRule`], [`AssignmentRule`] and [`RateRule`]; a document
/// writes all of them as elements of those names inside `<listOfRules>`.
pub trait Rule: SBase {
    /// The `math` child of this rule element: the formula the rule applies.
    fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this rule as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a rule is
    /// `self.math().set(Math::new(document, node))`.
    fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}

/// A rule of any kind: an `algebraicRule`, an `assignmentRule` or a `rateRule`.
///
/// The three kinds are declared as three types ([`AlgebraicRule`], [`AssignmentRule`], [`RateRule`])
/// that all recognize the element name of this supertype, and the *first* of these names belongs to
/// one of them. The identity-derived constructor is therefore suppressed: `new_empty(document)`
/// here would silently build an `<algebraicRule>`, which is what [`AlgebraicRule::new_empty`] is
/// for. A rule whose kind the caller chooses is built through [`AbstractRule::default`], which
/// takes the element name explicitly.
///
/// ![UML][sbml-rule]
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["algebraicRule", "assignmentRule", "rateRule"],
    namespace = NS_SBML_CORE,
    constructor = false
)]
#[embed_doc_image("sbml-rule", "docs-images/uml-rule.svg")]
pub struct AbstractRule(XmlElement);

impl Rule for AbstractRule {}
impl XmlSupertype for AbstractRule {}

impl AbstractRule {
    /// The kind of this rule, as one of the four [`RuleTypes`].
    pub fn cast(self) -> RuleTypes {
        if let Some(rule) = self.try_downcast::<AlgebraicRule>() {
            RuleTypes::Algebraic(rule)
        } else if let Some(rule) = self.try_downcast::<AssignmentRule>() {
            RuleTypes::Assignment(rule)
        } else if let Some(rule) = self.try_downcast::<RateRule>() {
            RuleTypes::Rate(rule)
        } else {
            RuleTypes::Other(self)
        }
    }

    /// The default `<algebraicRule>` element of a document: the content the specification gives an empty one.
    pub fn default(document: Document, tag_name: &str) -> Self {
        AbstractRule::new_empty_named(document, tag_name)
    }
}

/// The `<algebraicRule>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "algebraicRule", namespace = NS_SBML_CORE)]
pub struct AlgebraicRule(XmlElement);

impl Rule for AlgebraicRule {}

impl XmlDefault for AlgebraicRule {
    fn default(document: Document) -> Self {
        AlgebraicRule::new_empty(document)
    }
}

impl XmlNamedSubtype<AbstractRule> for AlgebraicRule {}

/// The `<assignmentRule>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "assignmentRule", namespace = NS_SBML_CORE)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl XmlNamedSubtype<AbstractRule> for AssignmentRule {}

impl AssignmentRule {
    /// Creates a new `<assignmentRule>` element with the given `variable`.
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = AssignmentRule::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    /// The `variable` attribute of this `<assignmentRule>` element.
    pub fn variable(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("variable")
    }
}

/// The `<rateRule>` element of the SBML core specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "rateRule", namespace = NS_SBML_CORE)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl XmlNamedSubtype<AbstractRule> for RateRule {}

impl RateRule {
    /// Creates a new `<rateRule>` element with the given `variable`.
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = RateRule::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    /// The `variable` attribute of this `<rateRule>` element.
    pub fn variable(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("variable")
    }
}
