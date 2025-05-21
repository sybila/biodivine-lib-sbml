use crate::core::SBase;
use crate::xml::{OptionalXmlChild, RequiredXmlChild, RequiredXmlProperty};
use crate::Sbml;

#[test]
pub fn property_type() {
    let doc = Sbml::read_path("test-inputs/test-fbc/property_type_test.xml");
    let issues = doc.unwrap().validate();

    assert!(!issues.is_empty());
    assert_eq!(issues.get(0).unwrap().rule, "SANITY_CHECK")
}

#[test]
fn test_coefficient_special_values() {
    let doc = Sbml::read_path("test-inputs/test-fbc/special_values.xml").unwrap();

    let issues = doc.validate();

    assert!(!issues.is_empty());

    let flux_objective = doc
        .sbml_root()
        .model()
        .get()
        .unwrap()
        .objectives()
        .get()
        .unwrap()
        .get(0)
        .flux_objectives()
        .get()
        .get(0);

    assert_eq!(flux_objective.coefficient().get(), f64::INFINITY);
    assert_eq!(issues.get(0).unwrap().rule, "fbc-20608")
}
