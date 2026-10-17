use crate::Sbml;

#[test]
fn check_21204() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21204.xml");
    let issues = doc.unwrap().validate().unwrap();

    assert!(
        issues.iter().any(|issue| issue.rule() == "layout-21204"),
        "expected layout-21204 in {:?}",
        issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
    );
}
