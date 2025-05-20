use crate::core::sbase::SbmlUtils;
use crate::core::{Math, SBase, SId};
use crate::xml::{
    OptionalChild, RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlDocument, XmlElement,
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

    pub fn default(document: XmlDocument, tag_name: &str) -> Self {
        AbstractRule::new_empty(document, tag_name)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AlgebraicRule(XmlElement);

impl Rule for AlgebraicRule {}

impl XmlDefault for AlgebraicRule {
    fn default(document: XmlDocument) -> Self {
        AlgebraicRule::new_empty(document, "algebraicRule")
    }
}

impl XmlNamedSubtype<AbstractRule> for AlgebraicRule {
    fn expected_tag_name() -> &'static str {
        "algebraicRule"
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl XmlNamedSubtype<AbstractRule> for AssignmentRule {
    fn expected_tag_name() -> &'static str {
        "assignmentRule"
    }
}

impl AssignmentRule {
    pub fn new(document: XmlDocument, variable: &SId) -> Self {
        let obj = AssignmentRule::new_empty(document, "assignmentRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl XmlNamedSubtype<AbstractRule> for RateRule {
    fn expected_tag_name() -> &'static str {
        "rateRule"
    }
}

impl RateRule {
    pub fn new(document: XmlDocument, variable: &SId) -> Self {
        let obj = RateRule::new_empty(document, "rateRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("variable")
    }
}
