//! The document-wide pass that reports the value rules of the packages.
//!
//! The rules themselves are rows in [`crate::constants::package_value_rules`]; this module is the
//! walk that applies them. It is a pass over every element of the document rather than a step of the
//! type check, because the *detection* the type check performs runs once per accessor that reaches
//! an attribute (six times for a `qualitativeSpecies` of the corpus, measured) - which is invisible
//! while the report is the library's own lint and a duplicate-report problem the moment it is a
//! specification rule. A document has one of each element, so a pass over the document reports a
//! value once.
//!
//! Like the other lints, the pass runs for every document, whatever the type check found: a value
//! rule is about one attribute of one element, and it does not depend on the rest of the document
//! being type-consistent.

use crate::cancellation::CancellationCheck;
use crate::constants::package_value_rules::value_violations;
use crate::xml::XmlElement;
use crate::{SbmlIssue, SbmlResult};

/// Report every package attribute value the specification's own rule forbids.
pub(crate) fn validate_package_values(
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
    value_violations(element, issues);
    for child in element.child_elements() {
        walk_and_report(&child, issues, check)?;
    }
    Ok(())
}
