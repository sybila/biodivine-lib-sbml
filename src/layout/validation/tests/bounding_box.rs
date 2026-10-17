use crate::{Sbml, SbmlIssue};

/// The rule ids of a report, so a test asserts the rule it is about and not the exact size of a
/// report of a document that is minimal on purpose.
fn rules_of(issues: &[SbmlIssue]) -> std::collections::BTreeSet<&str> {
    issues.iter().map(|issue| issue.rule()).collect()
}

#[test]
fn check_layout_21305() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21305.xml");
    let issues: Vec<SbmlIssue> = doc.unwrap().validate().unwrap();

    assert!(
        rules_of(&issues).contains("layout-21305"),
        "expected layout-21305 in {:?}",
        rules_of(&issues)
    );
}

#[test]
fn check_layout_21703() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21303.xml");
    let issues: Vec<SbmlIssue> = doc.unwrap().validate().unwrap();

    assert!(
        rules_of(&issues).contains("layout-21703"),
        "expected layout-21703 in {:?}",
        rules_of(&issues)
    );
}
