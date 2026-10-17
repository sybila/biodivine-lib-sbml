//! The rule registry and the issue model of this crate.
//!
//! This module is not where the validation *logic* lives: the `validation` submodules of
//! `crate::core`, `crate::constraint`, `crate::qual` and `crate::layout` check the objects of the
//! core specification and of each supported package, and this module is what they report
//! through. It holds three things:
//!
//!  * [`RuleDescriptor`] — the metadata of one rule: its id, its kind, the severity of the issues
//!    it produces and a one-line summary. The descriptors of this crate are the table in
//!    [`rules`], so a rule id is written down once instead of at every call site that can report
//!    it.
//!  * [`RuleSet`] — a lookup from a rule id to its descriptor, with [`RuleSet::CORE`] as the table
//!    of this crate. A rule set is a value rather than a global, so an SBML extension can
//!    contribute its own rules next to the built-in ones and pass the result to
//!    [`SbmlIssue::new_in`].
//!  * [`SbmlIssue`] — the report a failed check produces, with the rule id, the kind and the
//!    severity it inherits from its descriptor.
//!
//! ## Specification rules and library lints
//!
//! Two kinds of issue exist and stay distinguishable ([`RuleKind`]):
//!
//!  * a **specification rule** is a rule of the SBML standard. Its id is the id the specification
//!    gives the rule: a five-digit number, optionally prefixed by the package that adds it (for
//!    example `10301` or `fbc-21207`). The SBML test suite compares against exactly these ids.
//!  * a **library lint** is a diagnostic this crate raises on its own initiative. No rule of the
//!    SBML standard requires it, so its id is not a specification rule id (a word, for example
//!    `unused-namespace`), and its severity is [`SbmlIssueSeverity::Info`] unless the diagnostic
//!    means that a typed read of the document is impossible.
//!
//! The distinction is data, not a convention: [`SbmlIssue::kind`] reports it, and the test-suite
//! harness uses it to keep a lint out of the comparison against the suite's expectation files,
//! which list specification rule ids only. Since a lint is not a rule of the standard, it cannot
//! make a document fail validation on its own either — it is an observation about the document,
//! not a verdict.

use std::borrow::Cow;

use crate::SbmlIssueSeverity;

mod issue;
mod rules;

pub use issue::SbmlIssue;

/// What an issue means: a rule of the SBML standard, or a diagnostic of this library.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum RuleKind {
    /// A rule of the SBML specification, identified by the specification's rule id.
    SpecRule,
    /// A diagnostic this library raises on its own initiative (a "library lint").
    LibraryLint,
}

/// The metadata of one rule: everything the crate knows about a rule id in one place.
///
/// The descriptor is the single source of truth for the id's [kind](RuleDescriptor::kind) and the
/// [severity](RuleDescriptor::severity) of the issues it produces, so a call site cannot choose
/// either of them.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RuleDescriptor {
    /// The rule id: the specification's rule id for a [`RuleKind::SpecRule`], and a word for a
    /// [`RuleKind::LibraryLint`].
    pub id: &'static str,
    /// Whether the rule is part of the SBML specification or a library lint.
    pub kind: RuleKind,
    /// The severity of every issue this rule produces.
    pub severity: SbmlIssueSeverity,
    /// A one-line summary of what the rule requires, adapted from the SBML specification.
    pub summary: &'static str,
}

impl RuleDescriptor {
    /// Creates a descriptor from its parts.
    ///
    /// This constructor exists for rule sets that are not part of this crate (an SBML extension
    /// can describe its rules this way); the descriptors of this crate are the table in
    /// [`RuleSet::CORE`], which writes the same information out once per rule.
    pub const fn new(
        id: &'static str,
        kind: RuleKind,
        severity: SbmlIssueSeverity,
        summary: &'static str,
    ) -> Self {
        RuleDescriptor {
            id,
            kind,
            severity,
            summary,
        }
    }
}

/// A set of [`RuleDescriptor`] objects, looked up by rule id.
///
/// The set keeps its rules in declaration order, which is what [`RuleSet::rules`] reports; there
/// is no other ordering, so two lookups of the same id always find the same descriptor. Build a
/// set with [`RuleSet::new`] to describe new rules, and combine it with an existing one through
/// [`RuleSet::with`].
#[derive(Debug, Clone)]
pub struct RuleSet {
    rules: Cow<'static, [RuleDescriptor]>,
}

impl RuleSet {
    /// Every rule id this crate can report.
    ///
    /// The table is the data in the `rules` submodule of this module; it is not built at runtime,
    /// so a lookup cannot fail for a rule id the crate emits.
    pub const CORE: &'static RuleSet = &RuleSet {
        rules: Cow::Borrowed(rules::CORE_TABLE),
    };

    /// Creates a rule set from an explicit list of descriptors.
    ///
    /// The descriptors keep the order they are given in.
    ///
    /// # Panics
    ///
    /// Panics if two descriptors share an id. Two descriptors for one id would make the severity
    /// of an issue depend on the lookup order, which is exactly what this module exists to
    /// prevent.
    pub fn new(rules: Vec<RuleDescriptor>) -> Self {
        let mut seen = Vec::with_capacity(rules.len());
        for rule in rules.iter() {
            if seen.contains(&rule.id) {
                panic!("Two rule descriptors share the id '{}'.", rule.id);
            }
            seen.push(rule.id);
        }
        RuleSet {
            rules: Cow::Owned(rules),
        }
    }

    /// The descriptor of `id`, or `None` if this set does not describe that id.
    pub fn get(&self, id: &str) -> Option<&RuleDescriptor> {
        self.rules.iter().find(|it| it.id == id)
    }

    /// Returns `true` if `id` denotes a specification rule rather than one of this set's library
    /// lints.
    ///
    /// This is the only classification of a rule id in the crate, so the answer cannot drift
    /// between its callers. An id the set does not know at all counts as a specification rule:
    /// the test-suite harness asks about the ids of the suite's own expectation files, which name
    /// specification rules this library may not implement (or even know) yet, and those
    /// expectations have to stay visible in the report. A *reported* issue can never be such an
    /// id, because [`SbmlIssue::new`] panics for an id that is not in [`RuleSet::CORE`].
    ///
    /// The harness asks this for both sides of the comparison — the issues the library reports
    /// and the issues the suite expects — because a `LibraryLint` must not be able to turn a
    /// document the suite expects to pass into a mismatch.
    pub fn is_spec_rule(&self, id: &str) -> bool {
        !matches!(self.get(id), Some(rule) if rule.kind == RuleKind::LibraryLint)
    }

    /// The descriptors of this set, in declaration order.
    pub fn rules(&self) -> &[RuleDescriptor] {
        &self.rules
    }

    /// Combines this set with `other`.
    ///
    /// The result holds the rules of this set, in their order, followed by the rules of `other`
    /// that this set does not already contain, in their order. A descriptor that both sets
    /// describe in the same way is not a conflict, so combining a set with itself is the set
    /// itself.
    ///
    /// # Panics
    ///
    /// Panics if both sets describe the same id differently. A conflicting descriptor would make
    /// the meaning of a rule id depend on which set is looked up first.
    pub fn with(&self, other: &RuleSet) -> RuleSet {
        let mut rules = self.rules.to_vec();
        for rule in other.rules.iter() {
            match self.get(rule.id) {
                None => rules.push(*rule),
                Some(existing) if existing == rule => {}
                Some(existing) => panic!(
                    "Conflicting descriptors for rule id '{}': {existing:?} and {rule:?}.",
                    rule.id
                ),
            }
        }
        RuleSet::new(rules)
    }

    /// The number of rules in this set.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns `true` if this set describes no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::XmlElement;

    /// An element to attach a test issue to; the issue model does not read it.
    fn element() -> XmlElement {
        let document = biodivine_lib_xml_dom::Document::parse_string("<sbml/>")
            .expect("The test document is well formed.");
        XmlElement::new(
            document
                .root()
                .expect("The test document has a root element."),
        )
    }

    /// A descriptor that this crate does not have, for the tests that need one.
    fn foreign_rule() -> RuleDescriptor {
        RuleDescriptor::new(
            "fbc-99999",
            RuleKind::SpecRule,
            SbmlIssueSeverity::Warning,
            "A test rule.",
        )
    }

    #[test]
    fn known_rule_id_takes_its_severity_and_kind_from_the_table() {
        let issue = SbmlIssue::new("10201", &element(), "Test message.");

        let descriptor = RuleSet::CORE
            .get("10201")
            .expect("The rule table contains rule 10201.");
        assert_eq!(issue.rule(), descriptor.id);
        assert_eq!(issue.severity(), descriptor.severity);
        assert_eq!(issue.kind(), descriptor.kind);

        let lint = SbmlIssue::new("unused-namespace", &element(), "Test message.");
        assert_eq!(lint.severity(), SbmlIssueSeverity::Info);
        assert_eq!(lint.kind(), RuleKind::LibraryLint);
    }

    #[test]
    #[should_panic(expected = "99999")]
    fn unknown_rule_id_panics() {
        SbmlIssue::new("99999", &element(), "Test message.");
    }

    #[test]
    fn rule_ids_are_unique_and_shaped_like_their_kind() {
        let mut seen = Vec::new();
        for rule in RuleSet::CORE.rules() {
            assert!(
                !seen.contains(&rule.id),
                "The rule id '{}' appears twice in the table.",
                rule.id
            );
            seen.push(rule.id);
            assert_eq!(
                is_specification_rule_id(rule.id),
                rule.kind == RuleKind::SpecRule,
                "The id '{}' does not have the shape of its kind {:?}.",
                rule.id,
                rule.kind
            );
        }
    }

    /// The specification's rule id shape: five digits, optionally prefixed by a package name.
    fn is_specification_rule_id(id: &str) -> bool {
        let number = match id.split_once('-') {
            Some((package, number)) => {
                if package.is_empty() || !package.chars().all(|it| it.is_ascii_lowercase()) {
                    return false;
                }
                number
            }
            None => id,
        };
        number.len() == 5 && number.chars().all(|it| it.is_ascii_digit())
    }

    #[test]
    fn rule_set_iteration_order_is_the_declaration_order() {
        let core = RuleSet::CORE;
        // Building the same set again from its own rules reproduces the order exactly.
        let rebuilt = RuleSet::new(core.rules().to_vec());
        assert_eq!(core.rules(), rebuilt.rules());

        // The rules are grouped: core, then each package, then the lints; within a group the rule
        // numbers increase strictly.
        let mut last_group: Option<String> = None;
        let mut last_number = 0;
        for rule in core.rules() {
            if rule.kind == RuleKind::LibraryLint {
                continue;
            }
            let (group, number) = match rule.id.split_once('-') {
                Some((package, number)) => (
                    package.to_string(),
                    number.parse::<u32>().expect("A numeric rule id."),
                ),
                None => (String::new(), rule.id.parse().expect("A numeric rule id.")),
            };
            if last_group.as_deref() != Some(group.as_str()) {
                last_group = Some(group);
            } else {
                assert!(
                    number > last_number,
                    "The rule id '{}' is out of order in the table.",
                    rule.id
                );
            }
            last_number = number;
        }
        assert_eq!(
            core.rules().last().map(|it| it.kind),
            Some(RuleKind::LibraryLint),
            "The library lints are the last rules of the table."
        );
    }

    #[test]
    fn spec_rule_classification_covers_unknown_ids() {
        let core = RuleSet::CORE;
        assert!(core.is_spec_rule("10201"));
        assert!(!core.is_spec_rule("SANITY_CHECK"));
        assert!(!core.is_spec_rule("unused-namespace"));
        // An id of the SBML test suite that this library does not implement.
        assert!(core.is_spec_rule("80701"));
    }

    #[test]
    fn combining_rule_sets_is_deterministic_and_rejects_conflicts() {
        let core = RuleSet::CORE;
        let foreign = RuleSet::new(vec![foreign_rule()]);

        assert_eq!(core.with(core).rules(), core.rules());
        let combined = core.with(&foreign);
        assert_eq!(combined.len(), core.len() + 1);
        assert_eq!(&combined.rules()[..core.len()], core.rules());
        assert_eq!(combined.rules().last(), Some(&foreign_rule()));
        assert_eq!(foreign.with(core).rules().last(), core.rules().last());

        let conflict = RuleSet::new(vec![RuleDescriptor::new(
            "10201",
            RuleKind::LibraryLint,
            SbmlIssueSeverity::Info,
            "A conflict.",
        )]);
        assert!(std::panic::catch_unwind(|| core.with(&conflict)).is_err());
    }

    #[test]
    #[should_panic(expected = "share the id")]
    fn duplicate_rule_ids_are_rejected() {
        RuleSet::new(vec![foreign_rule(), foreign_rule()]);
    }
}
