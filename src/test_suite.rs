use crate::{Sbml, SbmlIssueSeverity};
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
    test_helper("./syntactic", None, true, test_document);
}

/// A method that can be passed to `test_helper` to validate a document.
pub fn test_document(document: &str) -> Vec<(String, String)> {
    match Sbml::read_str(document) {
        Ok(doc) => doc
            .validate()
            .into_iter()
            .map(|issue| {
                let severity = match issue.severity {
                    SbmlIssueSeverity::Error => "Error",
                    SbmlIssueSeverity::Warning => "Warning",
                    SbmlIssueSeverity::Info => "Informational",
                };
                (issue.rule, severity.to_string())
            })
            .collect(),
        Err(_e) => {
            // This process *can* fail if the document is not encoded correctly.
            // In that case, rule 10101 should appear in the expected results. If it does
            // not appear in this list, it is an error.
            vec![("10101".to_string(), "Error".to_string())]
        }
    }
}
