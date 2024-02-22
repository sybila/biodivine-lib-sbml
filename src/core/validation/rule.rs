use crate::core::validation::{apply_rule_10102, apply_rule_10301, SbmlValidable};
use crate::core::{AbstractRule, Rule, RuleTypes, SBase};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlList, XmlWrapper};
use crate::{SbmlIssue, SbmlIssueSeverity};
use std::collections::HashSet;

impl SbmlValidable for AbstractRule {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl AbstractRule {
    /// ### Rule 10304
    /// The value of the attribute variable of every [AssignmentRule](crate::core::rule::AssignmentRule)
    /// and [RateRule](crate::core::rule::RateRule) objects must be unique across the set of all
    /// [AssignmentRule](crate::core::rule::AssignmentRule) and [RateRule](crate::core::rule::RateRule)
    /// objects in a model. In other words, a given model component cannot be the subject of both
    /// an assignment rule and a rate rule simultaneously.
    pub(crate) fn apply_rule_10304(
        list_of_rules: &XmlList<AbstractRule>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let mut variables: HashSet<String> = HashSet::new();

        for rule in list_of_rules.as_vec() {
            let variable = match rule.clone().cast() {
                RuleTypes::Assignment(rule) => rule.variable().get(),
                RuleTypes::Rate(rule) => rule.variable().get(),
                _ => continue,
            };

            if variables.contains(&variable) {
                issues.push(SbmlIssue {
                    element: rule.raw_element(),
                    message: format!(
                        "The variable ('{0}') of <{1}> is already present in the set of \
                        <assignmentRule> and <rateRule> objects.",
                        variable,
                        rule.tag_name()
                    ),
                    rule: "10304".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            } else {
                variables.insert(variable);
            }
        }
    }
}
