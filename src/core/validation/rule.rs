use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{AbstractRule, MetaId, Rule, RuleTypes, SId};
use crate::xml::{OptionalXmlChild, XmlList, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for AbstractRule {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
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
            // A rule whose `variable` is missing or unparsable is reported by the type check of
            // its own element, so this aggregation only compares the rules that have one.
            let variable = match rule.clone().cast() {
                RuleTypes::Assignment(rule) => XmlProperty::get_checked(&rule.variable()),
                RuleTypes::Rate(rule) => XmlProperty::get_checked(&rule.variable()),
                _ => continue,
            };
            let Ok(Some(variable)) = variable else {
                continue;
            };

            if variables.contains(&variable) {
                let tag_name = rule.xml_element().tag_name();
                let message = format!(
                    "The variable ('{variable}') of <{tag_name}> is already present in the set of \
                        <assignmentRule> and <rateRule> objects."
                );
                issues.push(SbmlIssue::new("10304", &rule, message));
            } else {
                variables.insert(variable);
            }
        }
    }
}
