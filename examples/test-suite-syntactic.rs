use biodivine_lib_sbml::{Sbml, SbmlIssue, SbmlIssueSeverity};
use std::collections::{HashMap, HashSet};
use std::path::Path;

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

    let result = test_inner(filter);

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

struct TestResults {
    error: Vec<String>,
    warning: Vec<String>,
    info: Vec<String>,
}

/// A helper functions that actually runs the test.
fn test_inner(filter: Option<HashSet<String>>) -> TestResults {
    let dir_path = "./syntactic";

    if !Path::new(dir_path).is_dir() {
        panic!("Test data is missing.")
    }

    if let Some(filter) = filter.as_ref() {
        println!(
            "Test suite restricted to {} rules: {:?}",
            filter.len(),
            filter
        );
    }

    let test_issue = |id: &str| {
        if let Some(filter) = filter.as_ref() {
            filter.contains(id)
        } else {
            true
        }
    };

    let mut tested = HashSet::new();

    let mut error_problems = Vec::new();
    let mut warning_problems = Vec::new();
    let mut info_problems = Vec::new();

    for rule_dir in std::fs::read_dir(dir_path).unwrap() {
        let rule_dir = rule_dir.unwrap();
        let name = rule_dir.file_name();
        let name = name.to_str().unwrap();
        if !rule_dir.path().is_dir() {
            println!("Skipping file {} (not a directory).", name);
            continue;
        }
        tested.insert(name.to_string());

        let mut test_cases = Vec::new();
        for test_file in std::fs::read_dir(rule_dir.path()).unwrap() {
            let test_file = test_file.unwrap();
            let test_name = test_file.file_name();
            let test_name = test_name.to_str().unwrap();
            if !test_name.ends_with(".xml") {
                continue;
            }
            if !test_name.contains("l3v1") {
                // Skip any tests that are not for SBML level 3 version 1.
                continue;
            }

            test_cases.push(test_name.to_string());
        }

        println!("Found {} test cases for rule {}.", test_cases.len(), name);

        for test_case in test_cases {
            let mut test_file = rule_dir.path();
            test_file.push(test_case.clone());
            let mut result_file = rule_dir.path();
            result_file.push(test_case.replace(".xml", ".txt"));

            println!(" > Testing {:?}", test_file);
            let mut expected = read_expected_issues(result_file.to_str().unwrap());

            let doc = Sbml::read_path(test_file.to_str().unwrap()).unwrap();
            let mut issues: Vec<SbmlIssue> = Vec::new();
            doc.validate(&mut issues);

            for issue in issues {
                if test_issue(issue.rule.as_str()) {
                    if expected.contains_key(&issue.rule) {
                        expected.remove(&issue.rule);
                    } else {
                        println!(
                            " >> Found issue {} that is not in the expected list: {}",
                            issue.rule, issue.message,
                        );
                        let report = format!(
                            "Test {}/{}: Found unexpected issue {} (severity {:?}).",
                            name, test_case, issue.rule, issue.severity
                        );
                        match issue.severity {
                            SbmlIssueSeverity::Error => error_problems.push(report),
                            SbmlIssueSeverity::Warning => warning_problems.push(report),
                            SbmlIssueSeverity::Info => info_problems.push(report),
                        };
                    }
                }
            }

            for (id, sev) in expected {
                if test_issue(id.as_str()) {
                    println!(" >> Missed expected issue {}.", id);
                    let report = format!(
                        "Test {}/{}: Missed issue {} (severity {:?}).",
                        name, test_case, id, sev,
                    );
                    match sev {
                        SbmlIssueSeverity::Error => error_problems.push(report),
                        SbmlIssueSeverity::Warning => warning_problems.push(report),
                        SbmlIssueSeverity::Info => info_problems.push(report),
                    };
                }
            }
        }
    }

    if let Some(filter) = filter {
        let missing = Vec::from_iter(filter.difference(&tested));
        println!(
            "WARNING: {} rules were requested but not found in the test suite: {:?}",
            missing.len(),
            missing
        );
    }

    TestResults {
        error: error_problems,
        warning: warning_problems,
        info: info_problems,
    }
}

fn read_expected_issues(result_file: &str) -> HashMap<String, SbmlIssueSeverity> {
    // TODO:
    //  This doesn't really work if the issue appears in the file multiple times.
    //  But it seems that this is not a problem for the cases that we are testing at the moment?
    let content = std::fs::read_to_string(result_file).unwrap();
    let mut last_rule = None;
    let mut result = HashMap::new();
    for line in content.lines() {
        let split = Vec::from_iter(line.split(':'));
        if split.len() != 2 {
            continue;
        }
        if split[0].trim() == "Validation id" {
            assert!(last_rule.is_none());
            last_rule = Some(split[1].trim().to_string());
        }
        if split[0].trim() == "Severity" {
            assert!(last_rule.is_some());
            let s = match split[1].trim() {
                "Error" => SbmlIssueSeverity::Error,
                "Warning" => SbmlIssueSeverity::Warning,
                "Info" => SbmlIssueSeverity::Info,
                _ => {
                    panic!("Unknown severity {}", split[1].trim());
                }
            };
            result.insert(last_rule.as_ref().unwrap().clone(), s);
            last_rule = None;
        }
    }

    result
}
