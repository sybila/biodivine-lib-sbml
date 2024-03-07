use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct TestResults {
    pub error: Vec<String>,
    pub warning: Vec<String>,
    pub info: Vec<String>,
}

/// A helper function that actually runs the test.
///
/// You should give it:
///     - A path to the test inputs (typically `./syntactic`).
///     - An optional set of rules that should be reported in the output (all input files are
///       always executed, but only issues related to the given rules are reported; if `None`,
///       all issues are reported).
///     - If `all_versions` is set to true, the code will try to also run some of the tests for
///       older SBML versions. However, issues from these are never reported, as they are likely
///       incompatible, this is mostly to improve coverage of various edge cases and issues.
///
/// Finally, since we don't have access to the `Sbml` structure in this sub-project, we require
/// a `read_document` function that will actually validate the document for us. The function
/// should return a vector of pairs `(rule_id, severity)` indicating all issues that were found
/// in the document.
pub fn test_helper<F: Fn(&str) -> Vec<(String, String)>>(
    dir_path: &str,
    filter: Option<HashSet<String>>,
    all_versions: bool,
    read_document: F,
) -> TestResults {
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

    let filter_rule = |id: &str| {
        if let Some(filter) = filter.as_ref() {
            filter.contains(id)
        } else {
            true
        }
    };

    // Remember the rule IDs that actually appeared in at least one test. That way we can tell
    // if some of the rules that are *supposed* to be tested actually just don't appear in the
    // test suite at all.
    let mut tested = HashSet::new();

    // Save issues based on severity.
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
            if test_name.contains("l3v2") || (all_versions && test_name.contains("l3v1")) {
                // Only save tests which are for Level 3 Version 2, or Version 1 if we want to
                // also run older tests.
                test_cases.push(test_name.to_string());
            }
        }

        println!("Found {} test cases for rule {}.", test_cases.len(), name);

        for test_case in test_cases {
            let mut test_file = rule_dir.path();
            test_file.push(test_case.clone());
            let mut result_file = rule_dir.path();
            result_file.push(test_case.replace(".xml", ".txt"));

            if test_case.contains("l3v1") {
                println!(" > Testing translated {:?}", test_file);
                let test_content = std::fs::read_to_string(test_file).unwrap();
                // "Lift" version one to version two.
                let test_content = test_content.replace(
                    "http://www.sbml.org/sbml/level3/version1/core",
                    "http://www.sbml.org/sbml/level3/version2/core",
                );
                let issues = read_document(test_content.as_str());
                println!(
                    " >> Found {} issues in the translated document. Ignoring.",
                    issues.len()
                );
            } else {
                /*
                   Try to run the test and record all inconsistencies in output.
                */

                println!(" > Testing {:?}", test_file);
                let mut expected = read_expected_issues(result_file.to_str().unwrap());

                for rule_id in expected.keys() {
                    tested.insert(rule_id.clone());
                }

                let test_content = std::fs::read_to_string(test_file).unwrap();
                let issues = read_document(test_content.as_str());

                for (rule_id, severity) in issues {
                    if filter_rule(rule_id.as_str()) {
                        if let Some(entry) = expected.get_mut(&rule_id) {
                            entry.1 -= 1;
                        } else {
                            println!(
                                " >> Found issue {} that is not in the expected list.",
                                rule_id,
                            );
                            let report = format!(
                                "Test {}/{}: Found unexpected issue {} (severity {:?}).",
                                name, test_case, rule_id, severity
                            );
                            match severity.as_str() {
                                "Error" => error_problems.push(report),
                                "Warning" => warning_problems.push(report),
                                "Informational" => info_problems.push(report),
                                _ => panic!("Invalid severity {}", severity),
                            };
                        }
                    }
                }

                for (id, (sev, count)) in expected {
                    if count == 0 {
                        // All issues of this type have been discovered.
                        continue;
                    }
                    if filter_rule(id.as_str()) {
                        println!(" >> Missed expected issue {}.", id);
                        let report = format!(
                            "Test {}/{}: Missed issue {} (severity {:?}).",
                            name, test_case, id, sev,
                        );
                        match sev.as_str() {
                            "Error" => error_problems.push(report),
                            "Warning" => warning_problems.push(report),
                            "Informational" => info_problems.push(report),
                            _ => panic!("Invalid severity {}", sev),
                        };
                    }
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

fn read_expected_issues(result_file: &str) -> HashMap<String, (String, usize)> {
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
                "Error" => "Error".to_string(),
                "Warning" => "Warning".to_string(),
                "Informational" => "Informational".to_string(),
                _ => {
                    panic!("Unknown severity {}", split[1].trim());
                }
            };
            let key = last_rule.as_ref().unwrap().clone();
            let entry = result.entry(key);
            let value = entry.or_insert((s.clone(), 0));
            assert_eq!(value.0, s);
            value.1 += 1;
            last_rule = None;
        }
    }

    result
}
