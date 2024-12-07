use biodivine_lib_sbml::{Sbml, SbmlIssueSeverity};
use sbml_test_suite::test_helper;
use std::collections::HashSet;

/// This is an integration test that uses the examples from the SBML test suite
/// to validate the functionality of the library.
///
/// The test data can be downloaded here: https://github.com/sbmlteam/sbml-test-suite/releases
///
/// Specifically, the syntactic tests should be extracted into a `syntactic` directory
/// in the main folder of the repository.
///
/// Since it is not super easy to break down each case into separate test, we instead compile
/// a report of all violations that is printed at the end of the test.
///
/// If you only want to test a specific subset of rules, you can provide these as command line
/// arguments.
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let filter: Option<HashSet<String>> = if args.len() > 1 {
        Some(HashSet::from_iter(args.into_iter().skip(1)))
    } else {
        None
    };

    let result = test_helper("./syntactic", filter, true, test_document);

    let error_problems = result.error.clone();
    let warning_problems = result.warning.clone();
    let info_problems = result.info.clone();

    println!("Found:");
    println!(" > {} error issues.", error_problems.len());
    println!(" > {} warning issues.", warning_problems.len());
    println!(" > {} info issues.", info_problems.len());

    let errors = error_problems.join("\n");
    std::fs::write("test_suite_error.txt", errors).unwrap();

    let warning = warning_problems.join("\n");
    std::fs::write("test_suite_warning.txt", warning).unwrap();

    let infos = info_problems.join("\n");
    std::fs::write("test_suite_info.txt", infos).unwrap();

    println!("Report written.");

    assert!(error_problems.is_empty());
    assert!(warning_problems.is_empty());
    assert!(info_problems.is_empty());
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
