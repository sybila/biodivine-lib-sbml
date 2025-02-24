use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{AbstractRule, Rule, RuleTypes, SId};
use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlList, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for AbstractRule {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<String>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl CanTypeCheck for AbstractRule {}

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
        let mut variables: HashSet<SId> = HashSet::new();

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
