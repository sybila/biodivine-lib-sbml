use crate::{RuleSet, Sbml, SbmlError, SbmlIssueSeverity};
use sbml_test_suite::test_helper;

/// Allows us to run a "simplified" version of the test when using `cargo test --all-features`.
/// This is useful when computing code coverage, but otherwise will always pass. The test
/// that can actually fail is implemented as one of the examples.
///
/// It is recommended to run this with the `--release` flag, because the text replacement when
/// updating SBML versions can take a long time.
#[test]
#[cfg_attr(not(feature = "sbml_test_suite"), ignore)]
fn sbml_test_suite_syntactic() {
    test_helper(
        "./syntactic",
        None,
        true,
        |id| RuleSet::CORE.is_spec_rule(id),
        test_document,
    );
}

/// The message the harness prints when a document cannot be validated because a cancellation
/// trigger fired; see [`test_document`].
pub(crate) const NO_CANCELLATION_TRIGGER: &str =
    "the SBML test-suite harness registers no cancellation trigger";

/// A method that can be passed to `test_helper` to validate a document.
///
/// The harness is a gate, not an interactive operation: it compares the library's report against
/// the expectation files of the SBML test suite, which only makes sense if the report was actually
/// produced. A cancellation request that reaches the harness would therefore silently replace the
/// report of the affected documents — the read would be reported as the "document could not be
/// read" rule `10101`, and the validation would report nothing at all — and the gate would compare
/// something that is not a validation result. Both are turned into a hard failure instead, so that
/// a stray trigger can never hollow out the gate.
///
/// # Panics
///
/// Panics if the read or the validation of a document is cancelled.
pub fn test_document(document: &str) -> Vec<(String, String)> {
    let doc = match Sbml::read_str(document) {
        Ok(doc) => doc,
        Err(SbmlError::Cancelled(_)) => panic!("{NO_CANCELLATION_TRIGGER}"),
        Err(_e) => {
            // This process *can* fail if the document is not encoded correctly.
            // In that case, rule 10101 should appear in the expected results. If it does
            // not appear in this list, it is an error.
            return vec![("10101".to_string(), "Error".to_string())];
        }
    };
    doc.validate()
        .expect(NO_CANCELLATION_TRIGGER)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A one-rule-directory corpus that is removed when it goes out of scope.
    ///
    /// [`test_helper`] takes the path of a directory of rule directories, and every document in it
    /// is read, validated and compared against its `.txt` expectation file, so the smallest corpus
    /// the harness accepts is one directory that holds one Level 3 Version 2 document and its (here
    /// empty) expectation file.
    struct TempCorpus {
        root: PathBuf,
    }

    impl TempCorpus {
        fn new() -> TempCorpus {
            let root = std::env::temp_dir().join(format!(
                "biodivine-lib-sbml-harness-guard-{}",
                std::process::id()
            ));
            let rule = root.join("99999");
            std::fs::create_dir_all(&rule).expect("The temporary corpus directory is created.");
            std::fs::write(
                rule.join("99999-pass-00-01-l3v2.xml"),
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model/>
</sbml>"#,
            )
            .expect("The temporary document is written.");
            // The harness reads the expectation file before it reads the document, so it has to
            // exist; an empty file says that the suite expects no issues at all.
            std::fs::write(rule.join("99999-pass-00-01-l3v2.txt"), "")
                .expect("The temporary expectation file is written.");
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

    /// The message of a panic payload, for the assertions below.
    fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
        if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            "<the panic payload is not a string>".to_string()
        }
    }

    /// A cancellation trigger that reaches the harness is a hard failure, never an empty or a
    /// falsified report.
    ///
    /// Without the guard, a stray trigger would make the harness report the "document cannot be
    /// read" rule `10101` for every document (or nothing at all), and the gate would compare
    /// something that is not a validation result. The test registers an already-fired trigger,
    /// runs the real `test_helper` over a minimal corpus and asserts that the run aborts with the
    /// guard's message.
    ///
    /// The panic is caught *inside* the registered trigger, so that the trigger is popped from the
    /// thread-local stack normally: a fired trigger left behind by a panic would cancel every
    /// later test of this binary, which runs on the same thread.
    #[test]
    fn a_cancelled_document_fails_the_harness_loudly() {
        let corpus = TempCorpus::new();

        let trigger = cancel_this::CancelAtomic::new();
        trigger.cancel();

        let result = cancel_this::on_atomic(trigger, || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                test_helper(
                    corpus.path(),
                    None,
                    true,
                    |id| RuleSet::CORE.is_spec_rule(id),
                    test_document,
                )
            }));
            let payload = match outcome {
                Ok(_) => panic!("The harness must not survive a cancelled document."),
                Err(payload) => payload,
            };
            let message = panic_message(payload.as_ref());
            assert!(
                message.contains(NO_CANCELLATION_TRIGGER),
                "The harness failed for an unexpected reason: {message}"
            );
            Ok::<(), crate::SbmlError>(())
        });

        assert!(result.is_ok(), "The guard's panic must not escape the run.");
    }
}
