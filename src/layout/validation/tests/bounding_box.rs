use crate::{Sbml, SbmlIssue};

#[test]
fn check_layout_21305() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21305.xml");
    let issues: Vec<SbmlIssue> = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21305");
}

#[test]
fn check_layout_21703() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21303.xml");
    let issues: Vec<SbmlIssue> = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21703");
}
