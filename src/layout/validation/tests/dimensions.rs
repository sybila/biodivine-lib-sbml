use crate::Sbml;

#[test]
fn check_layout_21703() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21703.xml");
    let issues = doc.unwrap().validate().unwrap();

    assert!(
        issues.iter().any(|issue| issue.rule() == "layout-21703"),
        "expected layout-21703 in {:?}",
        issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
    );
}
