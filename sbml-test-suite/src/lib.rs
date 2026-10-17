use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The namespace of SBML Level 3 Version 1 Core.
const L3V1_CORE: &str = "http://www.sbml.org/sbml/level3/version1/core";
/// The namespace of SBML Level 3 Version 2 Core.
const L3V2_CORE: &str = "http://www.sbml.org/sbml/level3/version2/core";

/// A disagreement between the library and the expectation files of the suite that is accepted and
/// documented, instead of being counted as a mismatch.
///
/// The suite's expectation files list the issues its own reference implementation reports, so a
/// check of this library that is *correct* but stricter than that implementation makes the filtered
/// gate red without anything being wrong: the document does violate the rule, the suite just does
/// not list it (typically because the reference implementation stops at the first problem of a
/// document, or because the rule is not one it implements). Such a disagreement is recorded here,
/// one entry per rule and document, with the reason and the specification citation, so that it is
/// visible in the harness instead of hidden in a disabled rule id: the rule stays implemented and
/// enabled, the *unfiltered* dump still shows the reported issue, and only the comparison against
/// the suite's list is relaxed for exactly that pair.
#[derive(Debug, Clone, Copy)]
pub struct KnownDifference {
    /// The rule id the library reports.
    pub rule: &'static str,
    /// A substring of the `directory/file` name of the documents it applies to.
    pub document: &'static str,
    /// Why the library is right and the suite's list does not mention it, with the citation.
    pub reason: &'static str,
}

impl KnownDifference {
    /// Whether this entry covers a reported issue of a document.
    ///
    /// The document is matched **exactly**, against the `directory/file` id the reports use: a
    /// substring match would let an entry silently cover a different document whose name happens to
    /// contain the pattern, which is the opposite of what this list is for.
    pub fn covers(&self, rule: &str, document: &str) -> bool {
        self.rule == rule && self.document == document
    }
}

/// The result of a whole-corpus run of [`test_helper`].
pub struct TestResults {
    /// Mismatches where the library reported an issue the suite does not expect, by severity.
    pub error: Vec<String>,
    pub warning: Vec<String>,
    pub info: Vec<String>,
    /// The known differences that did **not** cover any reported issue in this run, by index.
    ///
    /// An entry of [`KnownDifference`] is a claim about a specific disagreement; if the library
    /// stops reporting that issue (because the check was weakened, or because the document is not
    /// in this corpus any more), the entry silently keeps the comparison relaxed for a case that no
    /// longer exists - which is exactly the property the list must not have. A run therefore
    /// reports its unused entries, and the caller is expected to treat a non-empty list as a
    /// failure (the example does, with the entry and its reason in the message).
    pub unused_known_differences: Vec<usize>,
    /// One `document|rule|severity` line for *every* issue the library reported, for every
    /// document that was read.
    ///
    /// This report is deliberately *not* filtered by the `is_spec_rule` predicate of
    /// [`test_helper`]: the mismatch reports cover the specification rules only (they compare
    /// against the suite's expectation files, which list specification rule ids), so a library
    /// lint appears here and nowhere else. The diagnostic of a lint has no other coverage.
    ///
    /// This also includes the Level 3 Version 1 documents, whose issues the mismatch reports
    /// deliberately ignore: the translated half of the corpus is invisible to every gate that
    /// compares the mismatch reports, so a dump of every reported issue is the only way to see a
    /// regression there. The list is unsorted; callers that compare it should sort it.
    pub issues: Vec<String>,
}

/// The documents of one rule directory of the syntactic corpus.
pub struct CorpusRule {
    /// The name of the rule directory (which is also the rule id the suite associates with it).
    pub name: String,
    /// The documents in the directory that the harness reads.
    pub documents: Vec<CorpusDocument>,
}

/// A single document of the syntactic corpus.
pub struct CorpusDocument {
    /// The rule directory the document belongs to.
    pub rule: String,
    /// The file name of the document.
    pub file: String,
    /// The content of the document; for a Level 3 Version 1 document this is the content after
    /// the level/version substitution described in [`corpus_documents`].
    pub content: String,
    /// True if [`content`](CorpusDocument::content) was translated from an older version.
    pub translated: bool,
}

impl CorpusDocument {
    /// The `rule/file` identifier under which every report refers to this document.
    pub fn id(&self) -> String {
        format!("{}/{}", self.rule, self.file)
    }

    /// The path of the file that lists the expected issues (the same file name with a `.txt`
    /// suffix).
    pub fn expectation_path(&self, dir_path: &str) -> String {
        let stem = self.file.replace(".xml", ".txt");
        format!("{dir_path}/{}/{}", self.rule, stem)
    }
}

/// Enumerate the documents of the syntactic test suite that the harness reads.
///
/// The corpus is a directory per rule, each containing one XML document per test case. Only the
/// `l3v2` documents are read as they are; with `all_versions`, the `l3v1` documents are read as
/// well, after substituting the Level 3 Version 1 Core namespace with the Level 3 Version 2 Core
/// one ("lifting" the document). The remaining documents (other SBML levels and versions) are
/// skipped.
///
/// The enumeration is silent: it reports the non-directory entries of `dir_path` (which are not
/// documents) and otherwise returns the documents for the caller to process. It is the single
/// place where the selection and translation rules of the corpus live, so every tool that walks
/// the corpus sees exactly the same documents.
///
/// # Panics
///
/// Panics if `dir_path` is not a directory.
pub fn corpus_documents(dir_path: &str, all_versions: bool) -> Vec<CorpusRule> {
    if !Path::new(dir_path).is_dir() {
        panic!("Test data is missing.")
    }

    let mut result = Vec::new();
    for rule_dir in std::fs::read_dir(dir_path).unwrap() {
        let rule_dir = rule_dir.unwrap();
        let name = rule_dir.file_name();
        let name = name.to_str().unwrap();
        if !rule_dir.path().is_dir() {
            println!("Skipping file {} (not a directory).", name);
            continue;
        }

        let mut documents = Vec::new();
        for test_file in std::fs::read_dir(rule_dir.path()).unwrap() {
            let test_file = test_file.unwrap();
            let test_name = test_file.file_name();
            let test_name = test_name.to_str().unwrap();
            if !test_name.ends_with(".xml") {
                continue;
            }
            if !test_name.contains("l3v2") && !(all_versions && test_name.contains("l3v1")) {
                // Only read tests which are for Level 3 Version 2, or Version 1 if we want to
                // also run older tests.
                continue;
            }
            let translated = test_name.contains("l3v1");
            let content = std::fs::read_to_string(rule_dir.path().join(test_name)).unwrap();
            let content = if translated {
                // "Lift" version one to version two.
                content.replace(L3V1_CORE, L3V2_CORE)
            } else {
                content
            };
            documents.push(CorpusDocument {
                rule: name.to_string(),
                file: test_name.to_string(),
                content,
                translated,
            });
        }

        result.push(CorpusRule {
            name: name.to_string(),
            documents,
        });
    }
    result
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
///
/// The mismatch reports compare against the specification rule ids of the suite's expectation
/// files, so they must not contain an issue that no rule of the standard requires. The caller
/// therefore supplies `is_spec_rule`, which classifies a reported rule id; only issues the
/// predicate accepts take part in the comparison. The complete issue list of
/// [`TestResults::issues`] is *not* filtered, because it is the only coverage that diagnostics
/// outside the specification have.
///
/// Besides the mismatch reports, the result carries a line for every reported issue of every
/// document (see [`TestResults::issues`]).
///
/// `known_differences` lists the disagreements that are accepted and documented (see
/// [`KnownDifference`]); an issue that one of them covers is reported in neither mismatch report.
pub fn test_helper<F: Fn(&str) -> Vec<(String, String)>>(
    dir_path: &str,
    filter: Option<HashSet<String>>,
    all_versions: bool,
    is_spec_rule: impl Fn(&str) -> bool,
    known_differences: &[KnownDifference],
    read_document: F,
) -> TestResults {
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

    // One line per issue the library reported, for every document that was read.
    let mut issues = Vec::new();

    // How many reported issues each known difference covered, so an entry that covers nothing can
    // be reported at the end of the run.
    let mut known_difference_matches = vec![0usize; known_differences.len()];

    let corpus = corpus_documents(dir_path, all_versions);
    for rule in &corpus {
        let name = rule.name.as_str();
        tested.insert(name.to_string());

        println!(
            "Found {} test cases for rule {}.",
            rule.documents.len(),
            name
        );

        for document in &rule.documents {
            let test_file: PathBuf = Path::new(dir_path).join(name).join(&document.file);

            if document.translated {
                println!(" > Testing translated {:?}", test_file);
                let reported = read_document(document.content.as_str());
                println!(
                    " >> Found {} issues in the translated document. Ignoring.",
                    reported.len()
                );
                let id = document.id();
                for (rule_id, severity) in reported {
                    issues.push(format!("{id}|{rule_id}|{severity}"));
                }
            } else {
                /*
                   Try to run the test and record all inconsistencies in output.
                */

                println!(" > Testing {:?}", test_file);
                let mut expected = read_expected_issues(&document.expectation_path(dir_path));

                for rule_id in expected.keys() {
                    tested.insert(rule_id.clone());
                }

                let reported = read_document(document.content.as_str());

                for (rule_id, severity) in reported.iter() {
                    if filter_rule(rule_id.as_str()) && is_spec_rule(rule_id.as_str()) {
                        if let Some(entry) = expected.get_mut(rule_id) {
                            entry.1 -= 1;
                        } else if let Some(index) = known_differences
                            .iter()
                            .position(|known| known.covers(rule_id, &document.id()))
                        {
                            known_difference_matches[index] += 1;
                            let known = &known_differences[index];
                            println!(
                                " >> Known difference: {} on {} - {}",
                                rule_id,
                                document.id(),
                                known.reason
                            );
                        } else {
                            println!(
                                " >> Found issue {} that is not in the expected list.",
                                rule_id,
                            );
                            let report = format!(
                                "Test {}/{}: Found unexpected issue {} (severity {:?}).",
                                name, document.file, rule_id, severity
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
                    if filter_rule(id.as_str()) && is_spec_rule(id.as_str()) {
                        println!(" >> Missed expected issue {}.", id);
                        let report = format!(
                            "Test {}/{}: Missed issue {} (severity {:?}).",
                            name, document.file, id, sev,
                        );
                        match sev.as_str() {
                            "Error" => error_problems.push(report),
                            "Warning" => warning_problems.push(report),
                            "Informational" => info_problems.push(report),
                            _ => panic!("Invalid severity {}", sev),
                        };
                    }
                }

                let id = document.id();
                for (rule_id, severity) in reported {
                    issues.push(format!("{id}|{rule_id}|{severity}"));
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

    let unused_known_differences = known_difference_matches
        .iter()
        .enumerate()
        .filter(|(_, matches)| **matches == 0)
        .map(|(index, _)| index)
        .collect();

    TestResults {
        error: error_problems,
        warning: warning_problems,
        info: info_problems,
        unused_known_differences,
        issues,
    }
}

/// Read the expected issues of one test case from its `.txt` file.
///
/// The file lists the expected result as `Validation id: <id>` followed by `Severity: <severity>`
/// pairs; the returned map counts how many issues of each rule the suite expects.
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
