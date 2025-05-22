use crate::Sbml;

#[test]
fn check_qual_20607() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20607.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20607")
}

#[test]
fn check_qual_20608() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20608.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 2);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20608")
}

#[test]
fn check_qual_20609() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20609.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20609")
}
