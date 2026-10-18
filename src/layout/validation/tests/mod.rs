mod bounding_box;
mod curve;
mod dimensions;
mod point;

use crate::Sbml;

#[test]
fn check_empty_list() {
    let doc = Sbml::read_path("test-inputs/test-layout/empty_list.xml");
    let issues = doc.unwrap().validate().unwrap();

    assert!(!issues.is_empty());
}
#[test]
fn check_layout_20310() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_20310.xml");
    let issues = doc.unwrap().validate().unwrap();

    assert!(
        issues.iter().any(|issue| issue.rule() == "layout-20310"),
        "expected layout-20310 in {:?}",
        issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
    );
}

#[test]
fn check_layout_20803() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_20803.xml");

    let issues = doc.unwrap().validate().unwrap();

    assert!(
        issues.iter().any(|issue| issue.rule() == "layout-20803"),
        "expected layout-20803 in {:?}",
        issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
    );
}

#[test]
fn check_compartment_glyph() {
    let doc = Sbml::read_path("test-inputs/test-layout/compartment_glyph_test.xml");

    let issues = doc.unwrap().validate().unwrap();

    assert_eq!(issues.len(), 2);
    assert_eq!(issues.first().unwrap().rule(), "layout-20509");
    assert_eq!(issues.get(1).unwrap().rule(), "layout-20508");
}

#[test]
fn check_reference_glyph() {
    let doc = Sbml::read_path("test-inputs/test-layout/reference_glyph_test.xml");
    let issues = doc.unwrap().validate().unwrap();

    assert!(issues.is_empty());
}
