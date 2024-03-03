use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309,
    apply_rule_10310, apply_rule_10312, SanityCheckable, SbmlValidable,
};
use crate::core::{AbstractRule, Rule, RuleTypes, SBase};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlList, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for AbstractRule {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for AbstractRule {}

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
                let tag_name = rule.tag_name();
                let message = format!(
                    "The variable ('{variable}') of <{tag_name}> is already present in the set of \
                        <assignmentRule> and <rateRule> objects."
                );
                issues.push(SbmlIssue::new_error("10304", &rule, message));
            } else {
                variables.insert(variable);
            }
        }
    }
}
