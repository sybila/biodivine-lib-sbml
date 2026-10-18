//! The document-wide pass that reports the value rules of the packages.
//!
//! The rules themselves are rows in [`crate::constants::package_rules`] (the attributes an element
//! permits, and the syntax of `fbc:chemicalFormula`) and in
//! [`crate::constants::package_value_rules`] (the values a package's data types allow); this module
//! is the walk that applies all three.
//!
//! It is a pass over every element of the document rather than a step of the type check, because a
//! type check of one element runs once per accessor that reaches it (twice for a `species` of the
//! corpus and six times for a `qualitativeSpecies`, both measured) - which is invisible while the
//! report is the library's own lint and a duplicate-report problem the moment it is a specification
//! rule. A document has one of each element, so a pass over the document reports each rule once.

use crate::cancellation::CancellationCheck;
use crate::constants::package_rules::{validate_chemical_formula, validate_package_attributes};
use crate::constants::package_value_rules::value_violations;
use crate::xml::XmlElement;
use crate::{SbmlIssue, SbmlResult};

/// Report every package rule violation of the document: the attributes an element does not permit,
/// the `chemicalFormula` values that are not formulas, and the values a package's data types reject.
pub(crate) fn validate_package_rules(
    root: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    walk_and_report(root, issues, check)?;
    Ok(())
}

fn walk_and_report(
    element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    check.tick()?;
    validate_package_attributes(element, issues);
    validate_chemical_formula(element, issues);
    value_violations(element, issues);
    for child in element.child_elements() {
        walk_and_report(&child, issues, check)?;
    }
    Ok(())
}
