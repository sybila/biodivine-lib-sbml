use biodivine_lib_sbml::{Sbml, SbmlIssue};

// To run this example, execute `cargo run --example basic_example`.
// If you want to add command line arguments, you can use
// `cargo run --example basic_example -- ARG_1 ARG_2`.
// Note the use of `--` to indicate that ARG_x values are meant as arguments
// for the example binary, not for `cargo` itself.
fn main() {
    let doc =
        Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
    // let model = doc.model().get().unwrap();
    // Print the whole document:
    // println!("{}", model.read_doc().write_str().unwrap());
    let mut issues: Vec<SbmlIssue> = Vec::new();
    doc.validate(&mut issues);

    assert_eq!(issues.len(), 0);
}
