//! The `unmodelled-level` lint: the document declares a level and version this crate does not read.
//!
//! SBML has more levels and versions than any one library implements. A document that declares one
//! of them is a valid SBML document, and this crate parses it and can read its `level` and
//! `version` attributes - but no object model resolves for it, so *nothing* in it is checked, and a
//! report of "no issues" would say the opposite of what it means. The lint is how the crate says
//! what it did: the document was read, and its content was not validated because the crate has no
//! tables for its level and version.
//!
//! The lint fires for a document whose **root element is a published SBML root** (so rule 20101
//! does not already name the problem) but whose declared level and version is not a pair this crate
//! models. A root that is not a published SBML root at all is a *different* condition - the
//! document is not read as SBML, rule 20101 reports it, and the lint stays silent, so the two
//! cases remain distinguishable in a report.
//!
//! Like the other library lints of this crate, it is a
//! [`LibraryLint`](crate::validation::RuleKind::LibraryLint) of
//! [`Info`](crate::SbmlIssueSeverity::Info) severity: no rule of the SBML standard requires the check, and the document
//! is not invalid because of it. See [`crate::validation`] for the two kinds of issue.

use crate::cancellation::CancellationCheck;
use crate::constants::version::{
    document_level_version, root_is_published_sbml_root, LevelVersion,
};
use crate::xml::XmlElement;
use crate::{SbmlIssue, SbmlResult};

/// Reports the `unmodelled-level` lint for a document whose level and version this crate does not
/// model.
///
/// # Errors
///
/// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of the
/// current thread has fired.
pub(crate) fn validate_unmodelled_level(
    root: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    check.tick()?;
    if !root_is_published_sbml_root(root) || document_level_version(root).is_some() {
        return Ok(());
    }

    let level = root.get_attribute("level").unwrap_or_default();
    let version = root.get_attribute("version").unwrap_or_default();
    let modelled = LevelVersion::ALL
        .iter()
        .map(|it| format!("Level {} Version {}", it.level(), it.version()))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "The document declares SBML Level {level} Version {version}, which this library does not \
         model ({modelled} are modelled), so no object model resolves for it and its content is not \
         validated. Only the rules about the document root are checked."
    );
    issues.push(SbmlIssue::new("unmodelled-level", root, message));
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::validation::RuleKind;
    use crate::{RuleSet, SbmlIssueSeverity};

    /// The lint is a library lint of `Info` severity, so it cannot take part in the comparison
    /// against the SBML test suite: the suite's expectation files list specification rule ids, and
    /// a lint that classified as one would turn a document the suite expects to pass into a
    /// fictitious mismatch.
    #[test]
    fn the_lint_is_a_library_lint_of_info_severity() {
        let descriptor = RuleSet::CORE
            .get("unmodelled-level")
            .expect("the lint is registered in the rule table");
        assert_eq!(descriptor.kind, RuleKind::LibraryLint);
        assert_eq!(descriptor.severity, SbmlIssueSeverity::Info);
        assert!(
            !RuleSet::CORE.is_spec_rule("unmodelled-level"),
            "a lint is not a rule of the SBML standard"
        );
    }
}
