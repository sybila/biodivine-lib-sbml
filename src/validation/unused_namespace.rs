//! The `unused-namespace` library lint: a declared SBML namespace that nothing uses.
//!
//! A declaration such as `xmlns:layout="http://www.sbml.org/sbml/level3/version1/layout/version1"`
//! that no element or attribute of the document resolves through is dead weight: the document is
//! valid SBML without it, and no rule of the specification forbids it. The declaration is
//! therefore reported as a *library lint* ([`RuleKind::LibraryLint`](crate::RuleKind::LibraryLint)):
//! an observation about the document, not a verdict on it. Its severity is
//! [`Info`](crate::SbmlIssueSeverity::Info), so a caller that
//! only cares about conformance can ignore it by looking at [`SbmlIssue::kind`].
//!
//! ## What "used" and "scope" mean here
//!
//! The check is defined on the XML tree alone, which is the only thing the reader guarantees:
//!
//!  * a declaration belongs to the element that makes it, and its **scope** is that element's
//!    subtree. The check therefore looks at an element's own namespace declarations
//!    (`Element::namespace_declarations`), never at the bindings that are merely in scope
//!    (`Element::namespaces_in_scope`): a declaration is reported once, on the element that makes
//!    it, and never on a descendant that merely inherits it.
//!  * a declaration (prefix → URI) is **used** when some element or attribute in the declaring
//!    element's subtree resolves to that URI — the declaring element itself included, since its
//!    own name resolves through its own declaration. The test is on the URI, not on the prefix:
//!    a document that binds the same URI under two prefixes uses both declarations, which is the
//!    conservative answer (see below).
//!
//! The crate's table of conventional prefixes (`crate::constants::namespaces`) is deliberately
//! never consulted here. Reading is namespace-aware and prefix-agnostic, so a lint that reasoned
//! about prefixes would be able to contradict the reader; the URI is what both share.
//!
//! Two consequences are deliberate. First, only declarations of an SBML namespace
//! (`http://www.sbml.org/sbml/…`) are considered, because the routine declarations of an XML
//! namespace (`xmlns:xsi`, `xmlns:math`, `xmlns:xhtml`) and of the SBO vocabulary
//! (`xmlns:sbo`, `http://biomodels.net/sbo/`) are commonly present without a local use, and
//! reporting them would make the lint noise. Second, the check is conservative: when it cannot
//! decide — a URI that is used anywhere in the subtree — it stays silent, so a used declaration
//! can never be reported. Missing a dead declaration is the acceptable error here.
//!
//! ## Relationship to the rest of the namespace validation
//!
//! This lint is the *declared-but-unused* half of namespace validation. The other half — a name
//! whose namespace is not declared in scope, and a prefix that binds a used URI to the wrong
//! namespace — belongs to the XML layer ([`biodivine_lib_xml_dom::Document::validate`]) and is
//! reported by [`crate::Sbml::validate`] as a specification rule, so the namespace story is split
//! between the two and this module covers only its own half.

use std::collections::BTreeSet;

use crate::cancellation::CancellationCheck;
use crate::validation::SbmlIssue;
use crate::xml::XmlElement;
use crate::SbmlResult;

/// The prefix of every namespace the lint is responsible for: an SBML core namespace or the
/// namespace of an SBML package.
const SBML_NAMESPACE_PREFIX: &str = "http://www.sbml.org/sbml/";

/// The rule id of this lint, as registered in [`crate::RuleSet::CORE`].
const UNUSED_NAMESPACE: &str = "unused-namespace";

/// Reports every SBML namespace declaration that nothing in its scope uses.
///
/// The walk starts at `root` (the `<sbml>` element of a document) and visits every element once.
/// A single recursive pass collects the namespace URIs used by a subtree and reports the unused
/// declarations of the elements on the way back up, so the check is one traversal of the document
/// that accounts for one step per visited element, and it is therefore cancellable like every other
/// phase of [`crate::Sbml::validate`].
///
/// The reported issues are of kind [`RuleKind::LibraryLint`](crate::RuleKind::LibraryLint) with
/// severity [`Info`](crate::SbmlIssueSeverity::Info), as the rule table describes the id
/// [`UNUSED_NAMESPACE`].
///
/// # Errors
///
/// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of the
/// current thread has fired while the document was being walked.
pub(crate) fn validate_unused_namespaces(
    root: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    used_namespaces(root, issues, check)?;
    Ok(())
}

/// Reports the unused namespace declarations of `element` and returns the namespace URIs that
/// `element` and its descendants use.
///
/// One step of the walk per element, so that the lint pass of a large document is cancellable like
/// every other phase of [`crate::Sbml::validate`].
///
/// # Errors
///
/// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of the
/// current thread has fired.
fn used_namespaces(
    element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<BTreeSet<String>> {
    check.tick()?;

    let raw = element.element();
    let mut used = BTreeSet::new();
    if let Some(namespace) = raw.namespace() {
        used.insert(namespace.uri().to_string());
    }
    for name in raw.attributes().keys() {
        if let Some(namespace) = name.namespace() {
            used.insert(namespace.uri().to_string());
        }
    }
    for child in element.child_elements() {
        used.append(&mut used_namespaces(&child, issues, check)?);
    }

    for (prefix, declaration) in raw.namespace_declarations() {
        // `xmlns=""` undeclares the default namespace and binds no URI, so there is nothing that
        // could use it.
        let Some(namespace) = declaration else {
            continue;
        };
        let uri = namespace.uri();
        if !uri.starts_with(SBML_NAMESPACE_PREFIX) || used.contains(uri) {
            continue;
        }
        let declaration = match prefix.as_ref() {
            Some(prefix) => format!("xmlns:{prefix}"),
            None => "xmlns".to_string(),
        };
        let message = format!(
            "The namespace declaration [{declaration}] for '{uri}' on <{}> is not used by any \
             element or attribute in its scope.",
            element.full_name()
        );
        issues.push(SbmlIssue::new(UNUSED_NAMESPACE, element, message));
    }

    Ok(used)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{RuleKind, RuleSet};
    use crate::{Sbml, SbmlIssueSeverity};

    /// The `unused-namespace` issues of a document, in the order the document reports them.
    ///
    /// The tests of this module are about this lint, so they filter by rule id: a synthetic
    /// document may well violate specification rules as well (its elements are minimal on purpose),
    /// and it may report the other library lint too (a document whose part is not type-consistent
    /// is reported as `not-validated`), and neither must decide whether the lint is reported.
    fn lint_issues(document: &str) -> Vec<SbmlIssue> {
        Sbml::read_str(document)
            .expect("The test document is well formed.")
            .validate()
            .unwrap()
            .into_iter()
            .filter(|issue| issue.rule() == "unused-namespace")
            .collect()
    }

    /// A document whose `<model>` contains `content`, in the Level 3 Version 2 core namespace.
    fn document(declarations: &str, content: &str) -> String {
        format!(
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" {declarations}
                    level="3" version="2"><model id="m">{content}</model></sbml>"#
        )
    }

    const LAYOUT: &str = "http://www.sbml.org/sbml/level3/version1/layout/version1";

    #[test]
    fn declaration_used_by_an_element_name_is_not_reported() {
        let document = document(&format!(r#"xmlns:my="{LAYOUT}""#), "<my:layout/>");

        assert!(lint_issues(&document).is_empty());
    }

    #[test]
    fn declaration_used_by_an_attribute_name_is_not_reported() {
        let document = document(
            &format!(r#"xmlns:my="{LAYOUT}""#),
            r#"<parameter id="p" my:name="x"/>"#,
        );

        assert!(lint_issues(&document).is_empty());
    }

    #[test]
    fn declaration_used_below_the_declaring_element_is_not_reported() {
        // The declaration is on `<model>`, the element that uses it is a descendant of `<model>`.
        let document = format!(
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
                 <model id="m" xmlns:my="{LAYOUT}">
                   <listOfCompartments><my:compartmentGlyph/></listOfCompartments>
                 </model></sbml>"#
        );

        assert!(lint_issues(&document).is_empty());
    }

    #[test]
    fn declaration_of_a_used_uri_under_a_second_prefix_is_not_reported() {
        // The core URI is used by every element of the document, so the second binding of that URI
        // is not reported either: the check is conservative and reasons about URIs.
        let document = document(
            r#"xmlns:core="http://www.sbml.org/sbml/level3/version2/core""#,
            "",
        );

        assert!(lint_issues(&document).is_empty());
    }

    #[test]
    fn unused_declaration_is_reported_as_an_info_lint() {
        let document = document(&format!(r#"xmlns:dead="{LAYOUT}""#), "");

        let issues = lint_issues(&document);
        assert_eq!(issues.len(), 1);
        let issue = issues.first().unwrap();
        assert_eq!(issue.rule(), UNUSED_NAMESPACE);
        assert_eq!(issue.severity(), SbmlIssueSeverity::Info);
        assert_eq!(issue.kind(), RuleKind::LibraryLint);
        assert!(
            issue.message().contains("xmlns:dead"),
            "{}",
            issue.message()
        );
        assert!(issue.message().contains(LAYOUT), "{}", issue.message());
        assert_eq!(issue.element().local_name().as_str(), "sbml");
        // The severity and the kind are read from the produced issue, so the registered descriptor
        // is load-bearing.
        assert_eq!(
            RuleSet::CORE.get(UNUSED_NAMESPACE).map(|it| it.severity),
            Some(SbmlIssueSeverity::Info)
        );
    }

    #[test]
    fn unused_declaration_on_a_nested_element_is_reported() {
        // The shape of the `rdf:RDF` declaration that the annotation of many validated models
        // carries: a nested element declares an SBML namespace that only the annotation's own
        // vocabulary uses.
        let document = document(
            "",
            r#"<notes><body xmlns:sbml="http://www.sbml.org/sbml/level2/version3" xmlns="http://www.w3.org/1999/xhtml"><p/></body></notes>"#,
        );

        let issues = lint_issues(&document);
        assert_eq!(issues.len(), 1);
        let issue = issues.first().unwrap();
        assert_eq!(issue.rule(), UNUSED_NAMESPACE);
        assert_eq!(issue.element().local_name().as_str(), "body");
        assert!(
            issue.message().contains("xmlns:sbml"),
            "{}",
            issue.message()
        );
    }

    #[test]
    fn unused_default_declaration_is_reported() {
        // A default declaration is used by the declaring element itself unless that element is in
        // another namespace: here the root is in `http://www.some`, so the core declaration that
        // the document additionally makes is dead.
        let document = r#"<other:sbml xmlns="http://www.sbml.org/sbml/level3/version2/core"
                                       xmlns:other="http://www.some" level="3" version="2"/>"#;

        let issues = lint_issues(document);
        assert_eq!(issues.len(), 1);
        let issue = issues.first().unwrap();
        assert!(issue.message().contains("[xmlns]"), "{}", issue.message());
    }

    #[test]
    fn other_namespaces_are_never_reported() {
        // The routine declarations of a document that uses annotations or MathML: none of them is
        // an SBML namespace, so none of them is this lint's business.
        for (prefix, uri) in [
            ("xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ("sbo", "http://biomodels.net/sbo/"),
            ("math", "http://www.w3.org/1998/Math/MathML"),
            ("xhtml", "http://www.w3.org/1999/xhtml"),
        ] {
            let document = document(&format!(r#"xmlns:{prefix}="{uri}""#), "");
            assert!(
                lint_issues(&document).is_empty(),
                "The declaration of {uri} must never be reported."
            );
        }
    }
}
