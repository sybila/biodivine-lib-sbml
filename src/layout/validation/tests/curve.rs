use crate::Sbml;

#[test]
fn check_layout_10402() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_10402.xml");
    let issues = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    let issue = issues.iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-10402");
}
