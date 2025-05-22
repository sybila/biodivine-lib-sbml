use crate::Sbml;

#[test]
fn check_qual_20308() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20308.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20308")
}

#[test]
fn check_qual_20309() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20309.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20309")
}

#[test]
fn check_qual_20310() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20310.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 2);
    assert_eq!(issues.get(1).unwrap().rule, "qual-20310")
}
