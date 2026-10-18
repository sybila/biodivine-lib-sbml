//! Runs the extension example, and pins the guarantee the test-suite harness needs from an injected
//! rule set.
//!
//! The example in `examples/extension-example.rs` is the reference for writing an SBML package and
//! the part of the Biodivine book about extensions includes it verbatim. Including the *source* in a
//! test is what keeps that reference honest: the code the book shows is compiled and run here, on
//! every `cargo test`.
//!
//! The second test is about the harness rather than about the package: a document whose report
//! carries an injected rule must leave the invariants of the test-suite comparison intact. Those
//! invariants are data this crate established long before extensions existed —
//!
//!  * the *census* (the comparison against the suite's expectation files) is filtered to
//!    specification rules, so a library lint of a package cannot turn a document the suite expects to
//!    pass into a mismatch, while a rule the package's specification defines *must* be visible in it;
//!  * the *issue dump* is deliberately unfiltered, so a report the comparison cannot express (a lint,
//!    or an issue beyond what an expectation file lists) is still recorded.
//!
//! The test runs the real harness ([`sbml_test_suite::test_helper`]) over a one-document corpus with
//! the package's rules injected, and asserts both halves: the package's specification rule appears
//! in the comparison, its lint does not, and the dump keeps both.

// The example is a binary (`fn main`), so it is included as a module rather than imported. Its
// items are `pub`, which is what lets this file call `main` directly.
#[path = "../examples/extension-example.rs"]
mod extension_example;

use std::collections::HashSet;
use std::path::PathBuf;

use biodivine_lib_sbml::core::Model;
use biodivine_lib_sbml::extension::{ExtensionCast, ExtensionOf, ExtensionValidation};
use biodivine_lib_sbml::xml::OptionalXmlChild;
use biodivine_lib_sbml::{RuleSet, Sbml, SbmlError, SbmlIssueSeverity, URL_SBML_CORE};

use extension_example::{MyExtensionRules, MY_EXTENSION};

/// A view whose package URL is *not* shaped like the namespace of a package of the standard.
///
/// The cast of this view succeeds on a document that declares the URL - the cast asks whether the
/// document declares the package, not what the package is called - while the *type check* of this
/// crate reports the content of that namespace (an element under rule `10102`, an attribute under
/// the rule of the element it is on). The test below pins both halves, because the second one is the
/// case an extension author runs into first.
struct ForeignNamespaceModel {
    model: Model,
}

impl ExtensionOf<Model> for ForeignNamespaceModel {
    const PACKAGE: (&'static str, &'static str) = ("foreign", "http://example.org/foreign");

    fn from_object(model: Model) -> Self {
        ForeignNamespaceModel { model }
    }

    fn object(&self) -> &Model {
        &self.model
    }
}

/// The example runs to completion: every step of the life cycle it demonstrates (the refused cast,
/// the activating cast, the values written and read back, the two severities of injected rule, the
/// round trip through a file) is an assertion inside it.
#[test]
fn the_extension_example_runs() {
    // The example's own entry point, so the test cannot pass on a path the binary does not take (and
    // so the included `main` is not dead code).
    extension_example::main();
}

/// A two-document corpus, both documents declaring the package and each with an empty expectation
/// file, so the suite expects nothing from either of them.
///
/// The directory is keyed on the process id, so one corpus per process is the assumption: a second
/// test that used a `TempCorpus` concurrently would share the path and the two would delete each
/// other's files.
///
/// The first document breaks the package's specification rule (a tuning parameter without a
/// `value`), and the second is valid but carries the package's diagnostic (an empty list): the two
/// behaviours the invariants below are about are therefore exercised by one run.
struct TempCorpus {
    root: PathBuf,
}

impl TempCorpus {
    fn new() -> TempCorpus {
        let root = std::env::temp_dir().join(format!(
            "biodivine-lib-sbml-extension-harness-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let rule = root.join("myext-10101");
        std::fs::create_dir_all(&rule).expect("The temporary corpus directory is created.");
        std::fs::write(
            rule.join("myext-10101-fail-01-01-l3v2.xml"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" xmlns:myext="{}" myext:required="false"
      level="3" version="2">
  <model>
    <myext:listOfTuningParameters>
      <myext:tuningParameter myext:id="tp1"/>
    </myext:listOfTuningParameters>
  </model>
</sbml>
"#,
                MY_EXTENSION.1
            ),
        )
        .expect("The temporary document is written.");
        std::fs::write(
            rule.join("myext-empty-list-pass-00-01-l3v2.xml"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" xmlns:myext="{}" myext:required="false"
      level="3" version="2">
  <model>
    <myext:listOfTuningParameters/>
  </model>
</sbml>
"#,
                MY_EXTENSION.1
            ),
        )
        .expect("The temporary document is written.");
        // The expectation files list nothing: the harness reads them before the documents.
        for document in [
            "myext-10101-fail-01-01-l3v2.txt",
            "myext-empty-list-pass-00-01-l3v2.txt",
        ] {
            std::fs::write(rule.join(document), "").expect("The expectation file is written.");
        }
        TempCorpus { root }
    }

    fn path(&self) -> &str {
        self.root
            .to_str()
            .expect("The temporary path is valid UTF-8.")
    }
}

impl Drop for TempCorpus {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Reads, validates (with the package's rules injected) and reports a document the way the harness
/// expects: one `(rule id, severity)` pair per issue, with the severity spelled as the suite spells
/// it.
///
/// A document the XML layer cannot parse is reported as `10101` rather than as a failure, which is
/// the convention of the suite and not a rule of this package: `10101` is the specification rule "an
/// SBML document must be well-formed XML", so the suite's expectation files express an unreadable
/// document that way, and the harness of this crate does the same (`src/test_suite.rs`).
fn validate_with_extension(
    content: &str,
    extension: &dyn ExtensionValidation,
) -> Vec<(String, String)> {
    let document = match Sbml::read_str(content) {
        Ok(document) => document,
        Err(SbmlError::Cancelled(_)) => panic!("The harness registers no cancellation trigger."),
        Err(_) => return vec![("10101".to_string(), "Error".to_string())],
    };
    document
        .validate_with(&[extension])
        .expect("The harness registers no cancellation trigger.")
        .into_iter()
        .map(|issue| {
            let severity = match issue.severity() {
                SbmlIssueSeverity::Error => "Error",
                SbmlIssueSeverity::Warning => "Warning",
                SbmlIssueSeverity::Info => "Informational",
            };
            (issue.rule().to_string(), severity.to_string())
        })
        .collect()
}

/// An injected rule set does not disturb the invariants of the test-suite comparison.
#[test]
fn an_injected_rule_set_keeps_the_harness_invariants_intact() {
    let corpus = TempCorpus::new();
    let package = MyExtensionRules::default();

    // The rules of the run, which is what a caller classifies the reported ids with.
    let rules_of_the_run = RuleSet::CORE.with(package.rules());
    let results = sbml_test_suite::test_helper(
        corpus.path(),
        None,
        false,
        |id| rules_of_the_run.is_spec_rule(id),
        &[],
        |content| validate_with_extension(content, &package),
    );

    // (1) The census comparison sees the package's *specification* rule: the document is reported as
    //     having an issue the suite does not expect, which is exactly what an injected rule of the
    //     standard has to be able to do.
    assert!(
        results
            .error
            .iter()
            .any(|line| line.contains("myext-10101")),
        "The package's specification rule appears in the comparison: {:?}",
        results.error
    );

    // (2) ... and it does NOT see the package's library lint, even though the same document carries
    //     one: a lint is not a rule of any specification, so it can never produce a mismatch.
    let mismatch_lines: Vec<String> = results
        .error
        .iter()
        .chain(results.warning.iter())
        .chain(results.info.iter())
        .cloned()
        .collect();
    assert!(
        mismatch_lines
            .iter()
            .all(|line| !line.contains("myext-empty-list")),
        "The package's lint stays out of the comparison: {mismatch_lines:?}"
    );

    // (3) The unfiltered issue dump keeps both, which is what makes the record of a run a superset of
    //     what the comparison can express: the second document's diagnostic is recorded although the
    //     comparison cannot see it.
    let dumped: HashSet<&str> = results
        .issues
        .iter()
        .filter_map(|line| line.split('|').nth(1))
        .collect();
    assert!(
        dumped.contains("myext-10101"),
        "The dump records the package's rule: {:?}",
        results.issues
    );
    assert!(
        dumped.contains("myext-empty-list"),
        "The dump records the package's lint as well: {:?}",
        results.issues
    );

    // (4) The classification the filter uses is the one the crate documents: with the table of the
    //     crate alone the package's lint is indistinguishable from a specification rule, which is why
    //     a run that injects a rule set has to classify with the combined set.
    assert!(RuleSet::CORE.is_spec_rule("myext-empty-list"));
    assert!(!rules_of_the_run.is_spec_rule("myext-empty-list"));
    assert!(rules_of_the_run.is_spec_rule("myext-10101"));
}

/// A package whose namespace URL is not shaped like a namespace of the standard makes the core type
/// check report every element and attribute it adds, while a package with a spec-shaped URL is not
/// reported at all: the contract [`ExtensionOf::PACKAGE`] documents, asserted rather than described.
///
/// The three cases are the ones an extension author meets, and the rule each one is reported under
/// is part of the contract: an element of an undefined namespace is rule `10102` itself, and an
/// attribute of one is reported under the rule this crate's tables give the element it is on.
#[test]
fn the_namespace_url_of_a_package_decides_what_the_core_type_check_reports() {
    let document = |url: &str, body: &str| {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="{URL_SBML_CORE}" xmlns:pkg="{url}" pkg:required="false" level="3" version="2">
{body}
</sbml>
"#
        )
    };
    let rules_of = |url: &str, body: &str| {
        let document = Sbml::read_str(&document(url, body)).expect("The document is well formed.");
        document
            .validate()
            .expect("No cancellation trigger is registered.")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect::<Vec<_>>()
    };

    // (a) An element of the package's namespace, with the URL the example's package declares:
    //     nothing is reported, and the cast to the package's view succeeds.
    let element = "  <model><pkg:listOfThings/></model>";
    assert!(rules_of(MY_EXTENSION.1, element).is_empty());
    let declared = Sbml::read_str(&document(MY_EXTENSION.1, element)).expect("Well-formed.");
    let model = declared.model().get().expect("The document has a model.");
    assert!(model.cast::<extension_example::MyExtensionModel>().is_ok());

    // (b) The same element with a foreign URL: the element is in an undefined SBML namespace, which
    //     is rule 10102. The cast still succeeds - it asks whether the document *declares* the
    //     package, not what the package's URL is shaped like.
    let reports = rules_of("http://example.org/foreign", element);
    assert!(
        reports.contains(&"10102".to_string()),
        "an element of an undefined SBML namespace is reported by 10102: {reports:?}"
    );
    let foreign =
        Sbml::read_str(&document("http://example.org/foreign", element)).expect("Well-formed.");
    let model = foreign.model().get().expect("The document has a model.");
    assert!(model.cast::<ForeignNamespaceModel>().is_ok());

    // (c) An attribute of a foreign namespace, on a core element: it is reported under the rule the
    //     crate's tables give that element (rule 20517 for a compartment), because a report of an
    //     element's attributes belongs to the element's own rule.
    let attribute = r#"  <model><listOfCompartments><compartment id="c" size="1" constant="true" pkg:x="1"/></listOfCompartments></model>"#;
    let reports = rules_of("http://example.org/foreign", attribute);
    assert!(
        reports.contains(&"20517".to_string()),
        "an attribute of an undefined SBML namespace on a compartment is reported by 20517: {reports:?}"
    );
    assert!(rules_of(MY_EXTENSION.1, attribute).is_empty());
}
