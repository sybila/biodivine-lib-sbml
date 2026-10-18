//! Project documentation: the specification this crate implements, the decisions behind it, and the
//! coverage of its rules.
//!
//! This module is documentation rather than API. It exists so that the reasoning behind the library
//! and the state of its specification coverage are reachable from the same place as the API reference,
//! instead of living only in files that a reader of the documentation would never find. The pages are
//! the repository's design documents, included verbatim with [`include_str!`], so a change to a
//! document cannot leave the API documentation behind - and the documents stay readable in a plain
//! editor or on a code host.
//!
//! | page | contents |
//! | --- | --- |
//! | [`specification`](crate::documentation::specification) | what SBML consists of (levels, versions, packages, the shape of a document), which module of this crate implements which part of it, and what the crate deliberately does not do |
//! | [`design`](crate::documentation::design) | why the library is built the way it is: the handle model, the validation abstraction, levels as deltas, the typed MathML layer, the extension cast, the declarations, namespaces, cancellation, the walk guard, and the policy that keeps the lint gate closed |
//! | [`rule_coverage`](crate::documentation::rule_coverage) | which specification rules this crate implements, per level, version and package, with the gaps named - generated from `spec-coverage/rules.csv`, so it cannot drift from the matrix |
//!
//! Every number in these pages is anchored to something the crate ships: the recorded baseline under
//! `scripts/full-corpus-baseline/`, the scripts under `scripts/`, the coverage matrix under
//! `spec-coverage/`, or the crate's own tests, and the page names the anchor next to the number. The
//! few numbers that were established once while the design was being worked out (the count of
//! documents that made the walk panic before the guard existed, for example) are marked as such and
//! name the gate that keeps them at their value today. Answers to "how do I write an SBML extension"
//! are not a page: the
//! reference is `examples/extension-example.rs`, which the [`extension`] module
//! documents, and it is compiled and run by `tests/extension-example.rs`.
//!
//! The attribution of the specification text these pages adapt is on the crate's main page and in
//! `README.md`.

/// What SBML consists of, and where this crate implements it.
#[doc = include_str!("../docs/design/specification.md")]
pub mod specification {}

/// Why the library is built the way it is.
#[doc = include_str!("../docs/design/decisions.md")]
pub mod design {}

/// Which specification rules this crate implements, per level and package, with the gaps named.
///
/// The page is generated: `scripts/gen_rule_coverage_doc.py` renders it from
/// `spec-coverage/rules.csv`, and `scripts/verify.sh` fails when the file is out of date.
#[doc = include_str!("../docs/design/rule-coverage.md")]
pub mod rule_coverage {}
