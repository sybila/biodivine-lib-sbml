use crate::Sbml;

/// A `fbc:associatedSpecies` attribute that does not name a species of the enclosing model is
/// reported as `fbc-21207`.
///
/// The document is inline because the rule is about a dangling reference, which is easier to read
/// here than in a fixture file: `fbc:associatedSpecies` names `S5`, and the model declares only
/// `s1`. The document is otherwise valid, so the rule under test is the only issue it can produce.
#[test]
fn check_fbc_21207() {
    let document = r#"<?xml version="1.0"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core"
      xmlns:fbc="http://www.sbml.org/sbml/level3/version1/fbc/version2"
      level="3" version="2" fbc:required="false">
  <model id="m" fbc:strict="false">
    <listOfCompartments>
      <compartment id="c" size="1" constant="true"/>
    </listOfCompartments>
    <listOfSpecies>
      <species id="s1" compartment="c" initialAmount="1" hasOnlySubstanceUnits="false"
               boundaryCondition="false" constant="false"/>
    </listOfSpecies>
    <fbc:listOfGeneProducts>
      <fbc:geneProduct fbc:id="g1" fbc:label="b1" fbc:associatedSpecies="S5"/>
    </fbc:listOfGeneProducts>
  </model>
</sbml>"#;

    let doc = Sbml::read_str(document).expect("The test document is well formed.");
    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-21207");
}
