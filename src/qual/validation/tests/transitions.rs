use crate::Sbml;

#[test]
fn check_valid() {
    let doc = Sbml::read_path("test-inputs/test-qual/basic_example.xml").unwrap();

    let issues = doc.validate();
    println!("{:#?}", issues);

    assert_eq!(issues.len(), 0);
}