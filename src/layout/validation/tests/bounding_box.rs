use crate::core::validation::type_check::CanTypeCheck;
use crate::{Sbml, SbmlIssue};
use crate::xml::OptionalXmlChild;

#[test]
fn check_21305() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21305.xml");
    let model = doc.unwrap().model().get().unwrap().layouts().get().unwrap().get(0);

    let mut issues: Vec<SbmlIssue> = Vec::new();

    model.type_check(&mut issues);

    println!("{:#?}", issues);

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21305");
}

#[test]
fn check_21303() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21303.xml");
    let model = doc.unwrap().model().get().unwrap().layouts().get().unwrap().get(0);

    let mut issues: Vec<SbmlIssue> = Vec::new();

    model.type_check(&mut issues);

    println!("{:#?}", issues);

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21303");
}