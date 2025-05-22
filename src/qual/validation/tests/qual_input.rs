use crate::Sbml;

#[test]
fn check_qual_20509() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20509.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20509")
}

#[test]
fn check_qual_20508() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20508.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20508")
}
