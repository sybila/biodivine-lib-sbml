use biodivine_lib_sbml::{RuleSet, Sbml, SbmlIssueSeverity};
use biodivine_lib_xml_dom::Document;
use sbml_test_suite::{corpus_documents, test_helper};
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
/// arguments. Two optional flags change what the example writes:
///
///  - `--dump-issues` writes `test_suite_issues.txt`, one `document|rule|severity` line for
///    every issue the library reports for every document of the corpus (including the translated
///    Level 3 Version 1 documents, whose issues the mismatch reports ignore). Unlike the mismatch
///    reports, this file is not filtered to specification rules, so it is the only report that
///    covers the library's own diagnostics (its lints).
///  - `--check-roundtrip` writes `test_suite_roundtrip.txt`, which lists every corpus document
///    whose text does not survive a read -> write -> read cycle unchanged.
///
/// The flags are removed from the rule filter, so `--dump-issues` on its own still runs the whole
/// corpus. A run without either flag leaves neither file behind.
///
/// The three mismatch reports cover specification rules only: a reported issue whose rule id the
/// crate's rule table calls a library lint is not compared against the suite's expectations, which
/// list rule ids of the specification.
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let dump_issues = args.iter().any(|it| it == "--dump-issues");
    let check_roundtrip = args.iter().any(|it| it == "--check-roundtrip");
    if args.iter().any(|it| it == "--help" || it == "-h") {
        println!("usage: test-suite-syntactic [--dump-issues] [--check-roundtrip] [RULE ...]");
        return;
    }
    let rules = args
        .into_iter()
        .filter(|it| !it.starts_with("--"))
        .collect::<Vec<_>>();
    let filter: Option<HashSet<String>> = if rules.is_empty() {
        None
    } else {
        Some(HashSet::from_iter(rules))
    };

    let result = test_helper(
        "./syntactic",
        filter,
        true,
        |id| RuleSet::CORE.is_spec_rule(id),
        test_document,
    );

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

    if dump_issues {
        // One line per reported issue, sorted so that the order of the corpus walk does not leak
        // into the file: the whole-corpus issue gate compares this file against a snapshot.
        let mut issues = result.issues.clone();
        println!("Found {} reported issues in total.", issues.len());
        issues.sort();
        std::fs::write("test_suite_issues.txt", issues.join("\n")).unwrap();
    }

    if check_roundtrip {
        write_roundtrip_report();
    }

    println!("Report written.");

    assert!(error_problems.is_empty());
    assert!(warning_problems.is_empty());
    assert!(info_problems.is_empty());
}

/// Check that the text of every corpus document survives a round trip through the library and
/// write the mismatches into `test_suite_roundtrip.txt`.
///
/// The comparison is between the text nodes of a direct XML parse of the document and the text
/// nodes of the same document after `Sbml::read_str` -> serialization -> parse. Both sides go
/// through the same XML parser, so anything the XML layer itself normalizes (character
/// references, line endings) cancels out, and a difference is a change the *library* made to the
/// document. An empty report therefore means that reading a document does not rewrite its text.
///
/// Documents that the XML layer cannot parse at all (they declare an encoding other than UTF-8)
/// and documents that the library refuses to read are counted and skipped: they are covered by
/// the issue dump, which reports the read failure as rule 10101.
fn write_roundtrip_report() {
    let mut mismatches = Vec::new();
    let mut documents = 0usize;
    let mut unparsable = 0usize;
    let mut unreadable = 0usize;

    for rule in corpus_documents("./syntactic", true) {
        for document in rule.documents {
            let id = document.id();
            let Ok(direct) = Document::parse_string(&document.content) else {
                unparsable += 1;
                continue;
            };
            let Ok(sbml) = Sbml::read_str(&document.content) else {
                unreadable += 1;
                continue;
            };
            let Ok(first) = sbml.to_xml_string() else {
                mismatches.push(format!("{id}|serialization-failed"));
                continue;
            };
            if sbml.to_xml_string().ok().as_deref() != Some(first.as_str()) {
                mismatches.push(format!("{id}|unstable-serialization"));
            }
            let Ok(reparsed) = Document::parse_string(&first) else {
                mismatches.push(format!("{id}|unparsable-output"));
                continue;
            };

            documents += 1;
            let expected = text_nodes(&direct);
            let found = text_nodes(&reparsed);
            if expected == found {
                continue;
            }
            match expected.iter().zip(found.iter()).position(|(a, b)| a != b) {
                Some(index) => mismatches.push(format!(
                    "{id}|text|{}|{}|{}",
                    index,
                    abbreviated(&expected[index]),
                    abbreviated(&found[index])
                )),
                None => mismatches.push(format!(
                    "{id}|text-count|{}|{}",
                    expected.len(),
                    found.len()
                )),
            }
        }
    }

    println!(
        "Round trip: {documents} documents checked, {unparsable} not parsable as XML, \
         {unreadable} not readable as SBML, {} with a changed text.",
        mismatches.len()
    );
    mismatches.sort();
    std::fs::write("test_suite_roundtrip.txt", mismatches.join("\n")).unwrap();
}

/// The text of every text node of a document, in document order.
fn text_nodes(document: &Document) -> Vec<String> {
    document
        .nodes()
        .into_iter()
        .filter_map(|node| node.text().map(|text| text.as_str().to_string()))
        .collect()
}

/// A one-line, length-limited rendering of a text node, for the round-trip report.
fn abbreviated(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    if escaped.chars().count() > 60 {
        let prefix = escaped.chars().take(60).collect::<String>();
        format!("{prefix}...({} chars)", escaped.chars().count())
    } else {
        escaped
    }
}

/// A method that can be passed to `test_helper` to validate a document.
pub fn test_document(document: &str) -> Vec<(String, String)> {
    match Sbml::read_str(document) {
        Ok(doc) => doc
            .validate()
            .into_iter()
            .map(|issue| {
                let severity = match issue.severity() {
                    SbmlIssueSeverity::Error => "Error",
                    SbmlIssueSeverity::Warning => "Warning",
                    SbmlIssueSeverity::Info => "Informational",
                };
                (issue.rule().to_string(), severity.to_string())
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
