use crate::Sbml;

#[test]
fn check_layout_21703() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21703.xml");
    let issues = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21703");
}
