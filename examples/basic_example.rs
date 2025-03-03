use biodivine_lib_sbml::core::{SBase, SId, Species};
use biodivine_lib_sbml::xml::{
    OptionalDynamicChild, OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty,
    XmlChildDefault, XmlElement, XmlList, XmlWrapper,
};
use biodivine_lib_sbml::Sbml;

// To run this example, execute `cargo run --example basic_example`.
// If you want to add command line arguments, you can use
// `cargo run --example basic_example -- ARG_1 ARG_2`.
// Note the use of `--` to indicate that ARG_x values are meant as arguments
// for the example binary, not for `cargo` itself.
fn main() {
    // let doc = Sbml::read_path("test-inputs/COVID19_immunotherapy_Mathematical_Model.xml").unwrap();
    // let doc = Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
    let doc = Sbml::read_path("./test-inputs/model.sbml").expect("This document is not valid XML.");

    // First, we want to know if the document we
    // just read is valid SBML.
    let issues = doc.validate();
    if !issues.is_empty() {
        // Note that these could be just warnings/notes.
        // You can check `SbmlIssue::severity` of each item
        // to detect fatal errors.
        eprintln!("This document has issues:");
        for issue in issues {
            eprintln!("{:?}", issue);
        }
    }

    let model = doc
        .model()
        .get()
        // This is strange but allowed by the specification.
        .expect("This document does not contain any model.");

    // Note that individual lists of model components
    // are also optional in the specification. Here,
    // an empty list is created if it does not exist.
    let species = model.species().get_or_create();
    let compartments = model.compartments().get_or_create();
    println!(
        "This model has {} compartments and {} species.",
        compartments.len(),
        species.len(),
    );

    // We can use `DynamicProperty` and `DynamicChild` to access
    // items that are not in the SBML core specification.
    let qual_namespace = "http://www.sbml.org/sbml/level3/version1/qual/version1";

    // For example, here, we are reading the list of qualitative species defined
    // in the sbml-qual package as a "generic" list of `XmlElement` objects.
    let qual_species: OptionalDynamicChild<XmlList<XmlElement>> =
        model.optional_child("listOfQualitativeSpecies", qual_namespace);
    println!(
        "This model has {} qualitative species.",
        qual_species.get_or_create().len(),
    );

    // We can also modify the model.

    // First, create a new instance of a `Species` object.
    let species_id = SId::try_from("sp_1").unwrap();
    let compartment_id = compartments.get(0).id().get();
    let s = Species::new(model.document(), &species_id, &compartment_id);
    let species_name = "MySpecies".to_string();
    s.name().set_some(&species_name);

    // Then, add it to the current list of species.
    species.push(s);
    assert_eq!(species.get(0).name().get(), Some(species_name));

    // Finally, we can print the model back as XML:
    let xml_string = doc.to_xml_string().expect("Encoding error.");

    println!("{} ... ", &xml_string[..200]);
}
