use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::{SBase, SId};
use crate::math::{Math, MathNode};
use crate::xml::{
    Document, OptionalChild, OptionalXmlChild, RequiredSbmlProperty, RequiredXmlProperty,
    XmlDefault, XmlElement, XmlNamedSubtype, XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};

pub enum RuleTypes {
    // Other is used to represent rules that are only defined in (hypothetical) SBML extensions
    // that are not covered by this library.
    Other(AbstractRule),
    Algebraic(AlgebraicRule),
    Assignment(AssignmentRule),
    Rate(RateRule),
}

pub trait Rule: SBase {
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
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["algebraicRule", "assignmentRule", "rateRule"],
    namespace = NS_SBML_CORE,
    constructor = false
)]
pub struct AbstractRule(XmlElement);

impl Rule for AbstractRule {}
impl XmlSupertype for AbstractRule {}

impl AbstractRule {
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

    pub fn default(document: Document, tag_name: &str) -> Self {
        AbstractRule::new_empty_named(document, tag_name)
    }
}

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

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "assignmentRule", namespace = NS_SBML_CORE)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl XmlNamedSubtype<AbstractRule> for AssignmentRule {}

impl AssignmentRule {
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = AssignmentRule::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "rateRule", namespace = NS_SBML_CORE)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl XmlNamedSubtype<AbstractRule> for RateRule {}

impl RateRule {
    pub fn new(document: Document, variable: &SId) -> Self {
        let obj = RateRule::new_empty(document);
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("variable")
    }
}
