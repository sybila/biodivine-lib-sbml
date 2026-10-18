//! The package rules that constrain the *value* of an attribute, one row per specification rule.
//!
//! A package states a data type (or a range, or an enumeration) for the attributes it defines. The
//! crate's element tables carry the type, so the *detection* already exists: the property read
//! fails and `type_check_of_property` reports the library's own `SANITY_CHECK`. That lint is not a
//! specification rule, so a document whose only problem is such a value is compared as a *miss* of
//! the specification's rule plus a lint the comparison does not count.
//!
//! This module supplies the missing half: one row per rule, with the constraint the rule states and
//! the specification's sentence for it, evaluated **once per element** - the property read happens
//! once per accessor that reaches the attribute (six times for a `qualitativeSpecies`, measured),
//! which is invisible while the report is a lint and a duplicate-report problem the moment it is a
//! specification rule.
//!
//! Two rules can constrain the same attribute where the specification states the type and the range
//! separately (`qual-20306` "must be of the data type integer" and `qual-20312` "must not be
//! negative" on `qual:initialLevel`). They are two rows, and the constraint decides which one a value
//! violates: a value that is not an integer violates the type row, a well-formed negative integer the
//! range row, and a well-formed value neither.

use crate::constants::namespaces::{
    namespace_of_prefix, separate_prefix_name, Namespace, NS_FBC, NS_LAYOUT, NS_QUAL,
};
use crate::xml::XmlElement;
use crate::SbmlIssue;

/// The constraint a rule states about an attribute's value.
///
/// [`satisfies`] answers "is the value outside what this rule allows", so the rows of one attribute
/// partition the values between them: a value that is not an integer at all belongs to the type row
/// and to no other.
pub(crate) enum ValueConstraint {
    /// The value must be an integer.
    Integer,
    /// The value must be an integer and negative. This is how the specification's "must not be
    /// negative" rule is expressed: the literal values the rule forbids are the negative integers.
    NegativeInteger,
    /// The value must be a floating point number.
    Double,
    /// The value must be `true` or `false`.
    Boolean,
    /// The value must be one of these literals, lower-cased before the comparison.
    OneOf(&'static [&'static str]),
}

/// One specification rule about the value of an attribute.
pub(crate) struct AttributeValueRule {
    /// The element the attribute belongs to.
    pub(crate) element: &'static str,
    /// The attribute, spelled with the package's conventional prefix (`qual:maxLevel`).
    pub(crate) attribute: &'static str,
    /// The package namespace the attribute lives in.
    pub(crate) package: Namespace,
    /// The specification rule.
    pub(crate) rule: &'static str,
    /// The specification's sentence for the rule, adopted as the head of the message.
    pub(crate) requirement: &'static str,
    /// What the rule requires of the value.
    pub(crate) constraint: ValueConstraint,
}

/// Every attribute-value rule of the packages, one row per specification rule.
pub(crate) const PACKAGE_VALUE_RULES: &[AttributeValueRule] = &[
    AttributeValueRule {
        element: "model",
        attribute: "fbc:strict",
        package: NS_FBC,
        rule: "fbc-20210",
        requirement: "The attribute 'fbc:strict' on the <model> object must have a value of data type Boolean.",
        constraint: ValueConstraint::Boolean,
    },
    AttributeValueRule {
        element: "objective",
        attribute: "fbc:type",
        package: NS_FBC,
        rule: "fbc-20505",
        requirement: "The attribute 'fbc:type' on an <objective> object must be of the data type FbcType and thus its value must be one of 'minimize' or 'maximize'.",
        constraint: ValueConstraint::OneOf(&["minimize", "maximize"]),
    },
    AttributeValueRule {
        element: "fluxObjective",
        attribute: "fbc:coefficient",
        package: NS_FBC,
        rule: "fbc-20607",
        requirement: "The value of the attribute 'fbc:coefficient' of a <fluxObjective> object must conform to the syntax of the SBML data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "point",
        attribute: "layout:x",
        package: NS_LAYOUT,
        rule: "layout-21204",
        requirement: "The attributes 'layout:x', 'layout:y' and 'layout:z' of a <point> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "point",
        attribute: "layout:y",
        package: NS_LAYOUT,
        rule: "layout-21204",
        requirement: "The attributes 'layout:x', 'layout:y' and 'layout:z' of a <point> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "point",
        attribute: "layout:z",
        package: NS_LAYOUT,
        rule: "layout-21204",
        requirement: "The attributes 'layout:x', 'layout:y' and 'layout:z' of a <point> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "dimensions",
        attribute: "layout:width",
        package: NS_LAYOUT,
        rule: "layout-21704",
        requirement: "The attributes 'layout:width', 'layout:height' and 'layout:depth' of a <dimensions> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "dimensions",
        attribute: "layout:height",
        package: NS_LAYOUT,
        rule: "layout-21704",
        requirement: "The attributes 'layout:width', 'layout:height' and 'layout:depth' of a <dimensions> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "dimensions",
        attribute: "layout:depth",
        package: NS_LAYOUT,
        rule: "layout-21704",
        requirement: "The attributes 'layout:width', 'layout:height' and 'layout:depth' of a <dimensions> element must be of the data type 'double'.",
        constraint: ValueConstraint::Double,
    },
    AttributeValueRule {
        element: "qualitativeSpecies",
        attribute: "qual:constant",
        package: NS_QUAL,
        rule: "qual-20304",
        requirement: "The attribute 'qual:constant' in <qualitativeSpecies> must be of the data type Boolean.",
        constraint: ValueConstraint::Boolean,
    },
    AttributeValueRule {
        element: "qualitativeSpecies",
        attribute: "qual:initialLevel",
        package: NS_QUAL,
        rule: "qual-20306",
        requirement: "The attribute 'qual:initialLevel' in <qualitativeSpecies> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "qualitativeSpecies",
        attribute: "qual:initialLevel",
        package: NS_QUAL,
        rule: "qual-20312",
        requirement: "The attribute 'qual:initialLevel' in <qualitativeSpecies> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
    AttributeValueRule {
        element: "qualitativeSpecies",
        attribute: "qual:maxLevel",
        package: NS_QUAL,
        rule: "qual-20307",
        requirement: "The attribute 'qual:maxLevel' in <qualitativeSpecies> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "qualitativeSpecies",
        attribute: "qual:maxLevel",
        package: NS_QUAL,
        rule: "qual-20313",
        requirement: "The attribute 'qual:maxLevel' in <qualitativeSpecies> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
    AttributeValueRule {
        element: "input",
        attribute: "qual:thresholdLevel",
        package: NS_QUAL,
        rule: "qual-20507",
        requirement: "The attribute 'qual:thresholdLevel' in <input> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "input",
        attribute: "qual:thresholdLevel",
        package: NS_QUAL,
        rule: "qual-20510",
        requirement: "The attribute 'qual:thresholdLevel' in <input> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
    AttributeValueRule {
        element: "output",
        attribute: "qual:outputLevel",
        package: NS_QUAL,
        rule: "qual-20606",
        requirement: "The attribute 'qual:outputLevel' in <output> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "output",
        attribute: "qual:outputLevel",
        package: NS_QUAL,
        rule: "qual-20610",
        requirement: "The attribute 'qual:outputLevel' in <output> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
    AttributeValueRule {
        element: "defaultTerm",
        attribute: "qual:resultLevel",
        package: NS_QUAL,
        rule: "qual-20704",
        requirement: "The attribute 'qual:resultLevel' in <defaultTerm> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "defaultTerm",
        attribute: "qual:resultLevel",
        package: NS_QUAL,
        rule: "qual-20705",
        requirement: "The attribute 'qual:resultLevel' in <defaultTerm> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
    AttributeValueRule {
        element: "functionTerm",
        attribute: "qual:resultLevel",
        package: NS_QUAL,
        rule: "qual-20805",
        requirement: "The attribute 'qual:resultLevel' in <functionTerm> must be of the data type integer.",
        constraint: ValueConstraint::Integer,
    },
    AttributeValueRule {
        element: "functionTerm",
        attribute: "qual:resultLevel",
        package: NS_QUAL,
        rule: "qual-20806",
        requirement: "The attribute 'qual:resultLevel' in <functionTerm> must not be negative.",
        constraint: ValueConstraint::NegativeInteger,
    },
];

/// Whether `value` satisfies `constraint`.
fn satisfies(constraint: &ValueConstraint, value: &str) -> bool {
    match constraint {
        ValueConstraint::Integer => value.parse::<i64>().is_ok(),
        // "Must not be negative" is violated by the values that *are* integers and are negative;
        // a value that is not an integer at all belongs to the type rule, which is why this is a
        // negative test rather than a positive one.
        ValueConstraint::NegativeInteger => !value.parse::<i64>().is_ok_and(|it| it < 0),
        ValueConstraint::Double => value.parse::<f64>().is_ok() || spec_double(value),
        ValueConstraint::Boolean => value == "true" || value == "false",
        ValueConstraint::OneOf(literals) => {
            let value = value.to_lowercase();
            literals.iter().any(|it| *it == value)
        }
    }
}

/// The `double` literals of the specification that Rust's parser does not take: the specification's
/// data type is XML Schema's `double`, which also admits `INF`, `-INF` and `NaN`.
fn spec_double(value: &str) -> bool {
    matches!(value, "INF" | "-INF" | "NaN")
}

/// Report every attribute of `element` whose value violates a rule of [`PACKAGE_VALUE_RULES`].
///
/// The caller is the document-wide pass of `validation::package_values`, which visits each element
/// once.
pub(crate) fn value_violations(element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let tag_name = element.tag_name();
    let attributes = element.attributes();
    for rule in PACKAGE_VALUE_RULES {
        if rule.element != tag_name.as_str() {
            continue;
        }
        let (prefix, name) = separate_prefix_name(rule.attribute);
        if rule.package.0 != prefix {
            continue;
        }
        let namespace = namespace_of_prefix(prefix);
        let Some(value) = attributes.iter().find_map(|(attribute, value)| {
            (attribute.local_name_str() == name
                && attribute
                    .namespace()
                    .is_some_and(|it| it.uri() == namespace.1))
            .then(|| value.as_str())
        }) else {
            continue;
        };
        if satisfies(&rule.constraint, value) {
            continue;
        }
        let message = format!(
            "{requirement} The value is [{value}].",
            requirement = rule.requirement
        );
        issues.push(SbmlIssue::new(rule.rule, element, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuleSet;
    use crate::Sbml;

    #[test]
    fn every_value_row_names_a_registered_rule_and_a_published_package_namespace() {
        let set = &RuleSet::CORE;
        for rule in PACKAGE_VALUE_RULES {
            assert!(
                set.get(rule.rule).is_some(),
                "The rule `{}` has no descriptor in the rule table.",
                rule.rule
            );
            let (prefix, name) = separate_prefix_name(rule.attribute);
            assert!(!prefix.is_empty(), "`{}` names no package", rule.attribute);
            assert!(!name.is_empty(), "`{}` names no attribute", rule.attribute);
            assert_eq!(
                rule.package.0, prefix,
                "`{}` is not an attribute of the package it names",
                rule.attribute
            );
        }
    }

    #[test]
    fn the_type_row_and_the_range_row_part_the_values_between_them() {
        // The same attribute carries two rules; the value decides which one is reported.
        let rules = |value: &str| {
            let document = Sbml::read_str(&document(value)).expect("well-formed XML");
            let mut reported = document
                .validate()
                .expect("validation runs")
                .into_iter()
                .map(|issue| issue.rule().to_string())
                .filter(|rule| rule.starts_with("qual-"))
                .collect::<Vec<_>>();
            reported.sort();
            reported
        };
        // A value that is not an integer violates the type rule, and only it.
        assert_eq!(rules("2.5"), vec!["qual-20307".to_string()]);
        assert_eq!(rules("string"), vec!["qual-20307".to_string()]);
        // A negative integer violates the range rule, and only it.
        assert_eq!(rules("-1"), vec!["qual-20313".to_string()]);
        // A well-formed value violates neither.
        assert!(rules("2").is_empty());
    }

    #[test]
    fn a_value_is_reported_once_per_element() {
        // The property read that detects the same problem runs once per accessor; the rule must be
        // reported once per element or the comparison sees duplicates.
        let document = Sbml::read_str(&document("2.5")).expect("well-formed XML");
        let reported = document
            .validate()
            .expect("validation runs")
            .into_iter()
            .filter(|issue| issue.rule() == "qual-20307")
            .count();
        assert_eq!(reported, 1);
    }

    /// A document whose qualitative species carries `maxLevel="{value}"`.
    fn document(value: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
               <sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3" version="1" xmlns:qual="http://www.sbml.org/sbml/level3/version1/qual/version1" qual:required="false">
                 <model id="m">
                   <listOfCompartments><compartment id="c" size="1" constant="true"/></listOfCompartments>
                   <listOfQualitativeSpecies>
                     <qualitativeSpecies qual:id="s" compartment="c" qual:maxLevel="{value}"/>
                   </listOfQualitativeSpecies>
                 </model>
               </sbml>"#
        )
    }
}
