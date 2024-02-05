use biodivine_lib_sbml::sbase::SBase;
use biodivine_lib_sbml::validation::SbmlIssue;
use biodivine_lib_sbml::xml::{OptionalXmlChild, OptionalXmlProperty};
use biodivine_lib_sbml::Sbml;

// To run this example, execute `cargo run --example basic_example`.
// If you want to add command line arguments, you can use
// `cargo run --example basic_example -- ARG_1 ARG_2`.
// Note the use of `--` to indicate that ARG_x values are meant as arguments
// for the example binary, not for `cargo` itself.
fn main() {
    let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
    let model = doc.model().get().unwrap();
    assert_eq!("model_id", model.id().get().unwrap().as_str());
    // Print the whole document:
    // println!("{}", model.read_doc().write_str().unwrap());
    let mut issues: Vec<SbmlIssue> = Vec::new();
    doc.validate(&mut issues);
}
