//! The issue a failed check reports.

use biodivine_lib_xml_dom::Element;

use crate::validation::{RuleKind, RuleSet};
use crate::xml::XmlWrapper;
use crate::SbmlIssueSeverity;

/// A single problem found in a document, together with the rule that found it.
///
/// An issue names the rule it belongs to ([`SbmlIssue::rule`]), the [kind](SbmlIssue::kind) of
/// that rule, the [severity](SbmlIssue::severity) the rule assigns to it, the XML
/// [element](SbmlIssue::element) it was found on and a human-readable
/// [message](SbmlIssue::message) that explains the problem.
///
/// The rule id, the kind and the severity are not construction parameters: an issue is built by
/// [`SbmlIssue::new`] (or [`SbmlIssue::new_in`] for a rule set of an extension), which reads them
/// from the [`RuleDescriptor`](crate::RuleDescriptor) of the rule. A rule id therefore cannot
/// acquire a severity that its descriptor does not give it, and it cannot be spelled differently
/// at two call sites.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SbmlIssue {
    element: Element,
    rule: &'static str,
    kind: RuleKind,
    severity: SbmlIssueSeverity,
    message: String,
}

impl SbmlIssue {
    /// Creates an issue that [`RuleSet::CORE`] attributes to `id`.
    ///
    /// The kind and the severity come from the descriptor of `id` in the core rule set.
    ///
    /// # Panics
    ///
    /// Panics if the core rule set has no rule `id`. A rule id that the table does not describe is
    /// a programming error: it would report an issue whose severity and kind are undefined, and
    /// the panic names the id so that the missing entry is easy to add. The rule table is checked
    /// to contain every id the crate emits; see the module documentation of the crate's rule
    /// registry.
    pub fn new<E: XmlWrapper, S: ToString>(id: &str, element: &E, message: S) -> SbmlIssue {
        Self::new_in(RuleSet::CORE, id, element, message)
    }

    /// Creates an issue that `rules` attributes to `id`.
    ///
    /// This is the constructor for rules that are not in [`RuleSet::CORE`]: pass the rule set that
    /// describes them (for example the combination of [`RuleSet::CORE`] with the rules of an SBML
    /// extension) together with the id of the rule.
    ///
    /// # Panics
    ///
    /// Panics if `rules` has no rule `id`, for the same reason as [`SbmlIssue::new`].
    pub fn new_in<E: XmlWrapper, S: ToString>(
        rules: &RuleSet,
        id: &str,
        element: &E,
        message: S,
    ) -> SbmlIssue {
        let rule = rules.get(id).unwrap_or_else(|| {
            panic!("The rule id '{id}' is not registered in the rule set used to report it.")
        });
        SbmlIssue {
            element: element.xml_element().element().clone(),
            rule: rule.id,
            kind: rule.kind,
            severity: rule.severity,
            message: message.to_string(),
        }
    }

    /// The id of the rule that found this issue.
    pub fn rule(&self) -> &'static str {
        self.rule
    }

    /// Whether the rule that found this issue is a specification rule or a library lint.
    pub fn kind(&self) -> RuleKind {
        self.kind
    }

    /// The severity the rule assigns to this issue.
    pub fn severity(&self) -> SbmlIssueSeverity {
        self.severity
    }

    /// A human-readable description of the problem.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The XML element the problem was found on.
    pub fn element(&self) -> &Element {
        &self.element
    }
}
