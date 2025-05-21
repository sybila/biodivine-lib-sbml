use crate::Sbml;

#[test]
fn check_fbc_21103() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_21103.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-21103");
}

#[test]
fn check_fbc_20908() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20908.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20908");
}
