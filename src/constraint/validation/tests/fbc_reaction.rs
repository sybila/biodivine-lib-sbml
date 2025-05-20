use crate::Sbml;

#[test]
fn check_fbc_20716() {
    let doc = Sbml::read_path("test-inputs/test-fbc/strict_model.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20716")
}

#[test]
fn check_fbc_20705() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20705.xml").unwrap();

    let issues = doc.validate();

    println!("{:?}", issues);

    assert_eq!(issues.get(0).unwrap().rule, "fbc-20705")
}

#[test]
fn check_fbc_20707() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20707.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20707")
}
#[test]
fn check_fbc_20708() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20708.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20708")
}
#[test]
fn check_fbc_20709() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20709.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20709")
}
#[test]
fn check_fbc_207011() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20711.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20711")
}
#[test]
fn check_fbc_207012() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20712.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20712")
}
#[test]
fn check_fbc_207013() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20713.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20713")
}
#[test]
fn check_fbc_207010() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20710.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20710")
}
