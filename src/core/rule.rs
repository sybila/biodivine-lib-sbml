use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::{Math, SBase, SId};
use crate::xml::{
    Document, OptionalChild, RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlElement,
    XmlNamedSubtype, XmlSupertype,
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
    fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["algebraicRule", "assignmentRule", "rateRule"],
    namespace = NS_SBML_CORE
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
        AbstractRule::new_empty(document, tag_name)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "algebraicRule", namespace = NS_SBML_CORE)]
pub struct AlgebraicRule(XmlElement);

impl Rule for AlgebraicRule {}

impl XmlDefault for AlgebraicRule {
    fn default(document: Document) -> Self {
        AlgebraicRule::new_empty(document, "algebraicRule")
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
        let obj = AssignmentRule::new_empty(document, "assignmentRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<SId> {
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
        let obj = RateRule::new_empty(document, "rateRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("variable")
    }
}
