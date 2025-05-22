use crate::Sbml;

#[test]
fn check_valid() {
    let doc = Sbml::read_path("test-inputs/test-qual/basic_example.xml").unwrap();

    let issues = doc.validate();
    println!("{:#?}", issues);

    assert_eq!(issues.len(), 0);
}

#[test]
fn check_qual_20409() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20409.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20409")
}

#[test]
fn check_qual_20409_b() {
    let doc = Sbml::read_path("test-inputs/test-qual/test_qual_20409_b.xml").unwrap();

    let issues = doc.validate();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.get(0).unwrap().rule, "qual-20409")
}
