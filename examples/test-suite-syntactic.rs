use biodivine_lib_sbml::{RuleSet, Sbml, SbmlError, SbmlIssueSeverity};
use biodivine_lib_xml_dom::Document;
use sbml_test_suite::{
    corpus_documents, test_helper, unread_levels, DifferenceDirection, KnownDifference,
};
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
///    every issue the library reports for every document of the corpus. Unlike the mismatch
///    reports, this file is not filtered to specification rules, so it is the only report that
///    covers the library's own diagnostics (its lints) and the reports that go beyond an
///    expectation file.
///  - `--report-panics` keeps the run going when the validation of a document panics and prints
///    one `PANICKED|<message>` line for it, so `scripts/corpus_panic_inventory.py` can report which
///    documents of the corpus cannot be validated at all.
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
        println!(
            "usage: test-suite-syntactic [--dump-issues] [--check-roundtrip] [--report-panics] [RULE ...]"
        );
        return;
    }
    let report_panics = args.iter().any(|it| it == "--report-panics");
    let rules = args
        .into_iter()
        .filter(|it| !it.starts_with("--"))
        .collect::<Vec<_>>();
    let filter: Option<HashSet<String>> = if rules.is_empty() {
        None
    } else {
        Some(HashSet::from_iter(rules))
    };

    // `--report-panics` is a diagnostic mode: it turns a panic inside the validation of one
    // document into one `PANICKED|<message>` line on standard output (attributed to the document by
    // the ` > Testing` line before it) and keeps the walk going, so that the whole corpus can be
    // measured instead of stopping at the first broken document.
    // Say which levels the run does not read, so a reader of a parity report cannot mistake the
    // corpus coverage for the whole syntactic suite.
    for (level, reason) in unread_levels() {
        println!("Not reading {level} documents: {reason}.");
    }

    let result = test_helper(
        "./syntactic",
        filter,
        true,
        |id| RuleSet::CORE.is_spec_rule(id),
        KNOWN_DIFFERENCES,
        move |document| {
            if !report_panics {
                return test_document(document);
            }
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let owned = document.to_string();
            let outcome = std::panic::catch_unwind(move || test_document(&owned));
            std::panic::set_hook(previous);
            match outcome {
                Ok(issues) => issues,
                Err(payload) => {
                    let message = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|it| it.to_string()))
                        .unwrap_or_else(|| "unknown panic".to_string());
                    println!("PANICKED|{message}");
                    vec![]
                }
            }
        },
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

    // An entry of the known-difference list is a claim about one specific disagreement. An entry
    // that covered nothing in this run is an error, not a no-op: it would keep the comparison
    // relaxed for a case that no longer exists (a weakened check, or a document that left the
    // corpus). The check runs before the reports are written, so the whole-corpus gate reports the
    // failure as an incomplete run instead of comparing a report that was produced under a stale
    // assumption.
    if !result.unused_known_differences.is_empty() {
        for index in &result.unused_known_differences {
            let entry = &KNOWN_DIFFERENCES[*index];
            println!(
                "UNUSED KNOWN DIFFERENCE: {} on {} - {}",
                entry.rule, entry.document, entry.reason
            );
        }
        panic!(
            "{} entry/entries of KNOWN_DIFFERENCES covered no reported issue in this run.",
            result.unused_known_differences.len()
        );
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
///
/// # Panics
///
/// Panics if any of the operations is cancelled: like [`test_document`], this report is a gate,
/// so a cancellation request must not be counted as a property of the corpus. Every step that
/// would otherwise silently count a document (parsing, reading, serializing, re-parsing) checks
/// for a cancellation first.
fn write_roundtrip_report() {
    let mut mismatches = Vec::new();
    let mut documents = 0usize;
    let mut unparsable = 0usize;
    let mut unreadable = 0usize;

    for rule in corpus_documents("./syntactic", true) {
        for document in rule.documents {
            let id = document.id();
            let direct = match Document::parse_string(&document.content) {
                Ok(direct) => direct,
                Err(biodivine_lib_xml_dom::XmlError::Cancelled(_)) => {
                    panic!("{NO_CANCELLATION_TRIGGER}")
                }
                Err(_) => {
                    unparsable += 1;
                    continue;
                }
            };
            let sbml = match Sbml::read_str(&document.content) {
                Ok(sbml) => sbml,
                Err(SbmlError::Cancelled(_)) => panic!("{NO_CANCELLATION_TRIGGER}"),
                Err(_) => {
                    unreadable += 1;
                    continue;
                }
            };
            let first = match sbml.to_xml_string() {
                Ok(first) => first,
                Err(SbmlError::Cancelled(_)) => panic!("{NO_CANCELLATION_TRIGGER}"),
                Err(_) => {
                    mismatches.push(format!("{id}|serialization-failed"));
                    continue;
                }
            };
            let second = sbml.to_xml_string();
            if let Err(SbmlError::Cancelled(_)) = second {
                panic!("{NO_CANCELLATION_TRIGGER}")
            }
            if second.ok().as_deref() != Some(first.as_str()) {
                mismatches.push(format!("{id}|unstable-serialization"));
            }
            let reparsed = match Document::parse_string(&first) {
                Ok(reparsed) => reparsed,
                Err(biodivine_lib_xml_dom::XmlError::Cancelled(_)) => {
                    panic!("{NO_CANCELLATION_TRIGGER}")
                }
                Err(_) => {
                    mismatches.push(format!("{id}|unparsable-output"));
                    continue;
                }
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

/// The disagreements between this library and the expectation files of the suite that are accepted,
/// with the reason and the citation for each.
///
/// An entry does not disable a rule: the rule stays implemented and enabled, the *unfiltered* issue
/// dump still contains the reported issue (which is what makes it visible in the recorded snapshot
/// of `scripts/full_corpus_parity.sh`), and only the comparison against the suite's own list is
/// relaxed, for exactly the rule and documents named here. Every entry is a case where the library
/// is right about the specification and the suite's reference implementation does not report the
/// violation, so the alternative to listing it would be to weaken a correct check.
const KNOWN_DIFFERENCES: &[KnownDifference] = &[
    KnownDifference {
        rule: "10215",
        document: "20212/20212-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document puts a stray <initialAssignment/> inside <listOfRules>, which the \
                 suite reports as 20212, and its <assignmentRule> refers to <ci> c </ci> - a \
                 compartment the model does not define. L3V2 rule 10215 requires every <ci> that is \
                 not the first child of an <apply> to name an existing model entity, so the report \
                 is correct and the expectation file (which lists only 20212) does not mention it.",
    },
    KnownDifference {
        rule: "10215",
        document: "20229/20229-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape as 20212-fail-01-02, in <listOfConstraints>: the constraint's <math> \
                 refers to an entity the model does not define, which L3V2 rule 10215 forbids.",
    },
    KnownDifference {
        rule: "10313",
        document: "20206/20206-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document puts a stray <assignmentRule/> inside <listOfFunctionDefinitions>, \
                 which the suite reports as 20206, and its <species> declares \
                 `substanceUnits=\"substance\"` while the model defines no unit with that \
                 identifier. L3V2 rule 10313 requires every unit identifier reference to name a \
                 unit definition or a base unit, so the report is correct and the expectation file \
                 (which lists only 20206) does not mention it.",
    },
    KnownDifference {
        rule: "10313",
        document: "20209/20209-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape as 20206-fail-01-02: a stray <assignmentRule/> inside \
                 <listOfUnitDefinitions> (rule 20209), and a species whose \
                 `substanceUnits=\"substance\"` names no unit definition of the model.",
    },
    KnownDifference {
        rule: "10313",
        document: "20214/20214-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape: a stray <assignmentRule/> inside <listOfReactions> (rule 20214), and a \
                 species whose `substanceUnits=\"substance\"` names no unit definition of the \
                 model.",
    },
    KnownDifference {
        rule: "10313",
        document: "20223/20223-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape: a stray <assignmentRule/> inside <listOfRules> (rule 20223), and a \
                 species whose `substanceUnits=\"substance\"` names no unit definition of the \
                 model.",
    },
    KnownDifference {
        rule: "10313",
        document: "20226/20226-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape: a stray <assignmentRule/> inside <listOfInitialAssignments> (rule \
                 20226), and a species whose `substanceUnits=\"substance\"` and whose compartment's \
                 `units=\"l\"` name no unit definition of the model.",
    },
    KnownDifference {
        rule: "10313",
        document: "20231/20231-fail-01-02-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "Same shape: a stray <assignmentRule/> inside <listOfConstraints> (rule 20231), and \
                 a species whose `substanceUnits=\"substance\"` names no unit definition of the \
                 model.",
    },
    KnownDifference {
        rule: "10215",
        document: "21172/21172-fail-01-03-sev2-l3v2.xml",
        direction: DifferenceDirection::Reported,
        reason: "The kinetic law has one <localParameter> and it has no `id` (the suite reports \
                 21172 for that), while the formula refers to <ci> k </ci>: the reference names no \
                 model entity and no local parameter, which L3V2 rule 10215 forbids. The expectation \
                 file lists 21172 only.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-10301/fbc-10301-fail-01-08-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20103/fbc-20103-fail-01-02-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20103/fbc-20103-pass-00-02-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20201/fbc-20201-fail-01-03-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20205/fbc-20205-pass-00-02-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20207/fbc-20207-fail-01-02-fbc-20208-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20209/fbc-20209-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20210/fbc-20210-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20301/fbc-20301-fail-01-02-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20506/fbc-20506-fail-01-03-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20506/fbc-20506-fail-01-04-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20509/fbc-20509-fail-01-04-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "fbc-20608/fbc-20608-fail-01-03-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document is a Flux Balance Constraints document that uses the element names of the \
                 package's *version 3* while declaring the version 2 namespace (`listOfFluxes` where \
                 `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng` defines `listOfFluxObjectives`), so the crate's \
                 report of the unknown child is right for the namespace the document declares.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-20703/layout-20703-fail-01-04-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21303/layout-21303-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21303/layout-21303-fail-01-04-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21403/layout-21403-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21405/layout-21405-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21405/layout-21405-fail-01-02-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21502/layout-21502-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "layout-21602/layout-21602-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "qual-20405/qual-20405-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports the child under the core rule 10102, the rule for an element no table \
                 of the document's namespace permits; the suite's file lists the Layout package's own \
                 rule for the same element instead.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-01-layout-10401-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-01-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-02-layout-10401-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-02-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-03-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "10102/10102-fail-02-04-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the Layout package's own rule id for the element the document gets \
                 wrong, and the suite's file lists the core rule 10102 *as well*; the two agree that the \
                 element is not one its parent permits.",
    },
    KnownDifference {
        rule: "10102",
        document: "multi-20306/multi-20306-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "multi-20406/multi-20406-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "multi-20409/multi-20409-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "multi-20412/multi-20412-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "spatial-22650/spatial-22650-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "spatial-22750/spatial-22750-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10102",
        document: "spatial-22850/spatial-22850-fail-01-01-10102-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's content is in a package this crate does not model (its namespaces are \
                 namespaces of the standard, and what they may contain is that package's business), so \
                 the crate does not report the unknown child the suite lists under 10102.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-20507/layout-20507-fail-01-01-layout-20508-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-20607/layout-20607-fail-01-01-layout-20608-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-20707/layout-20707-fail-01-01-layout-20708-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-20807/layout-20807-fail-01-01-layout-20808-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-20907/layout-20907-fail-01-01-layout-20908-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-21007/layout-21007-fail-01-01-layout-21008-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10310",
        document: "layout-21107/layout-21107-fail-01-01-layout-21108-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10310 for an `id` that does not conform to SId syntax (these \
                 documents carry Layout glyph ids that begin with a digit); the suite's file lists only \
                 the rule the document is about.",
    },
    KnownDifference {
        rule: "10215",
        document: "20212/20212-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document's formula refers to an entity the model does not define, which rule 10215 \
                 forbids; the suite's file lists only the rule the document is about. Adjudicated on the \
                 Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10215",
        document: "20229/20229-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document's formula refers to an entity the model does not define, which rule 10215 \
                 forbids; the suite's file lists only the rule the document is about. Adjudicated on the \
                 Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10215",
        document: "21103/21103-fail-01-20-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document's formula refers to an entity the model does not define, which rule 10215 \
                 forbids; the suite's file lists only the rule the document is about. Adjudicated on the \
                 Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10215",
        document: "21172/21172-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document's formula refers to an entity the model does not define, which rule 10215 \
                 forbids; the suite's file lists only the rule the document is about. Adjudicated on the \
                 Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "20222",
        document: "fbc-20209/fbc-20209-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has two `listOfEventAssignments` in one `event`, which rule 20222 forbids; \
                 the suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21209",
        document: "10307/10307-fail-01-34-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of a `trigger` (rule 21209; the Version 2 \
                 text is the 'at most one' half), and this document's trigger has none. The suite's file \
                 lists only the rule the document is about, because its reference implementation stops at \
                 that problem.",
    },
    KnownDifference {
        rule: "21209",
        document: "10307/10307-fail-01-35-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of a `trigger` (rule 21209; the Version 2 \
                 text is the 'at most one' half), and this document's trigger has none. The suite's file \
                 lists only the rule the document is about, because its reference implementation stops at \
                 that problem.",
    },
    KnownDifference {
        rule: "21209",
        document: "10307/10307-fail-01-36-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of a `trigger` (rule 21209; the Version 2 \
                 text is the 'at most one' half), and this document's trigger has none. The suite's file \
                 lists only the rule the document is about, because its reference implementation stops at \
                 that problem.",
    },
    KnownDifference {
        rule: "21209",
        document: "10307/10307-fail-01-41-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of a `trigger` (rule 21209; the Version 2 \
                 text is the 'at most one' half), and this document's trigger has none. The suite's file \
                 lists only the rule the document is about, because its reference implementation stops at \
                 that problem.",
    },
    KnownDifference {
        rule: "21213",
        document: "10305/10305-fail-01-05-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21213",
        document: "10306/10306-fail-01-05-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21213",
        document: "10307/10307-fail-01-34-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21213",
        document: "10307/10307-fail-01-35-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21213",
        document: "10307/10307-fail-01-36-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21213",
        document: "10307/10307-fail-01-41-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `eventAssignment` (rule 21213; the \
                 Version 2 text is the 'at most one' half), and this document's event assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "21210",
        document: "10307/10307-fail-01-36-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of a `delay` (rule 21210; the Version 2 text \
                 is the 'at most one' half), and this document's delay has none. The suite's file lists \
                 only the rule the document is about.",
    },
    KnownDifference {
        rule: "20201",
        document: "10103/10103-fail-02-26-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has two `model` elements, which rule 20201 forbids; the suite's file lists \
                 only the rule the document is about.",
    },
    KnownDifference {
        rule: "20201",
        document: "20201/20201-fail-01-07-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document has two `model` elements; the crate reports the rule of the element that is \
                 duplicated rather than 20201, which the suite lists for the same defect.",
    },
    KnownDifference {
        rule: "10101",
        document: "01013/01013-fail-01-11-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10101 for a `notes` element that is not in the XHTML namespace; \
                 the suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-01-layout-10401-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-01-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-02-layout-10401-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-02-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-03-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "10102/10102-fail-02-04-layout-10402-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20206/20206-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20209/20209-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20214/20214-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20223/20223-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20226/20226-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10313",
        document: "20231/20231-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document refers to a unit the model does not define, and the crate checks that \
                 reference (rule 10313); the suite's file lists only the rule the document is about, \
                 because its reference implementation does not resolve unit references. Adjudicated on \
                 the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "20804",
        document: "10308/10308-fail-01-16-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of an `initialAssignment` (rule 20804; the \
                 Version 2 text is the 'at most one' half), and this document's initial assignment has \
                 none. The suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "20907",
        document: "10304/10304-fail-01-08-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of every rule object (rule 20907; the \
                 Version 2 text is the 'at most one' half), and this document's rule has none. The \
                 suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "20907",
        document: "10306/10306-fail-01-05-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "Version 1 requires exactly one `math` child of every rule object (rule 20907; the \
                 Version 2 text is the 'at most one' half), and this document's rule has none. The \
                 suite's file lists only the rule the document is about.",
    },
    KnownDifference {
        rule: "10216",
        document: "10216/10216-pass-00-09-sev1-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10216 for a local parameter whose identifier is used outside its \
                 `kineticLaw`; the suite's file for this *pass* document does not list it, so the \
                 document is one the reference implementation accepts and the crate does not. Adjudicated \
                 as a documented difference on the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10216",
        document: "10216/10216-pass-00-10-sev1-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10216 for a local parameter whose identifier is used outside its \
                 `kineticLaw`; the suite's file for this *pass* document does not list it, so the \
                 document is one the reference implementation accepts and the crate does not. Adjudicated \
                 as a documented difference on the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10216",
        document: "10216/10216-pass-00-11-sev1-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The crate reports rule 10216 for a local parameter whose identifier is used outside its \
                 `kineticLaw`; the suite's file for this *pass* document does not list it, so the \
                 document is one the reference implementation accepts and the crate does not. Adjudicated \
                 as a documented difference on the Level 3 Version 2 half by the earlier unit.",
    },
    KnownDifference {
        rule: "10201",
        document: "10201/10201-fail-01-18-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the MathML placement of these documents under the rules of the \
                 elements that carry the content rather than under 10201, which the suite lists as well.",
    },
    KnownDifference {
        rule: "10201",
        document: "10201/10201-fail-01-19-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The crate reports the MathML placement of these documents under the rules of the \
                 elements that carry the content rather than under 10201, which the suite lists as well.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21503/layout-21503-fail-01-05-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21503/layout-21503-fail-01-06-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21602/layout-21602-pass-00-01-sev1-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-03-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-04-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-05-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-06-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21603/layout-21603-fail-01-07-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "layout-21604/layout-21604-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "qual-20206/qual-20206-fail-01-01-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "10307",
        document: "qual-20206/qual-20206-fail-01-02-sev2-l3v1.xml",
        direction: DifferenceDirection::Reported,
        reason: "The document has a `metaid` that is already present in the model, which the crate \
                 reports as rule 10307; the suite's file lists only the rule the document is about, \
                 because its reference implementation stops at the document's first problem.",
    },
    KnownDifference {
        rule: "21226",
        document: "01016/01016-fail-01-01-21226-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The value of `persistent` in this document is not a boolean, which the crate reports as \
                 its own diagnostic (`SANITY_CHECK`, because the typed view of the attribute is unusable) \
                 rather than under the specification's type rule 21226.",
    },
    KnownDifference {
        rule: "21226",
        document: "01016/01016-fail-01-02-21226-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The value of `persistent` in this document is not a boolean, which the crate reports as \
                 its own diagnostic (`SANITY_CHECK`, because the typed view of the attribute is unusable) \
                 rather than under the specification's type rule 21226.",
    },
    KnownDifference {
        rule: "layout-21007",
        document: "layout-21007/layout-21007-fail-01-01-layout-21008-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "The document's `layout:speciesReference` value is `1SpeciesReference_Glucose`, which \
                 is not a readable SId, so the two rules of that attribute - its syntax (21007) and \
                 the reference it has to name (21008) - cannot be evaluated on it; the crate reports \
                 10310 for the identifier's syntax instead.",
    },
    KnownDifference {
        rule: "layout-21008",
        document: "layout-21007/layout-21007-fail-01-01-layout-21008-sev2-l3v1.xml",
        direction: DifferenceDirection::Missed,
        reason: "Same document and same value as the entry above: the reference rule cannot be \
                 evaluated on a value that is not an identifier, and the crate's `layout-21008` \
                 reports are the documents whose value is one - `layout-21008-fail-01-01-sev2-l3v1` \
                 is the one that keeps reporting the rule after the modifier-referent fix.",
    },
];

/// The message this example prints when a document cannot be validated because a cancellation
/// trigger fired; see [`test_document`].
const NO_CANCELLATION_TRIGGER: &str =
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
