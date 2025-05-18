mod bounding_box;
mod curve;
mod dimensions;
mod point;

use crate::Sbml;

#[test]
fn check_empty_list() {
    let doc = Sbml::read_path("test-inputs/test-layout/empty_list.xml");
    let issues = doc.unwrap().validate();

    assert!(!issues.is_empty());
}
#[test]
fn check_layout_20310() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_20310.xml");
    let issues = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-20310");
}

#[test]
fn check_layout_20803() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_20803.xml");

    let issues = doc.unwrap().validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule.as_str(), "layout-20803");
}

#[test]
fn check_compartment_glyph() {
    let doc = Sbml::read_path("test-inputs/test-layout/compartment_glyph_test.xml");

    let issues = doc.unwrap().validate();

    assert_eq!(issues.len(), 2);
    assert_eq!(issues.get(0).unwrap().rule.as_str(), "layout-20509");
    assert_eq!(issues.get(1).unwrap().rule.as_str(), "layout-20508");
}

#[test]
fn check_reference_glyph() {
    let doc = Sbml::read_path("test-inputs/test-layout/reference_glyph_test.xml");
    let issues = doc.unwrap().validate();

    assert!(issues.is_empty());
}
