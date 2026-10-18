use biodivine_lib_sbml::core::Species;
use biodivine_lib_sbml::xml::{OptionalXmlChild, RequiredXmlProperty, XmlChild};
use biodivine_lib_sbml::Sbml;

fn main() {
    let document = Sbml::read_str(
        r#"
        <sbml xmlns="http://www.sbml.org/sbml/level2/version4" level="2" version="4">
          <model>
            <listOfCompartments><compartment id="c" size="1"/></listOfCompartments>
            <listOfSpecies><species id="s" compartment="c" initialAmount="1"/></listOfSpecies>
          </model>
        </sbml>
    "#,
    )
    .unwrap();
    let model = document.model().get().unwrap();
    let species: Species = model.species().get().unwrap().get(0);
    println!("constant = {}", species.constant().get());
    assert!(!species.constant().get());
    println!("VALIDATION.md example verified");
}
