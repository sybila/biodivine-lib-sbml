[![Crates.io](https://img.shields.io/crates/v/biodivine-lib-sbml?style=flat-square)](https://crates.io/crates/biodivine-lib-sbml)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-lib-sbml/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/sybila/biodivine-lib-sbml/build.yml?branch=main&style=flat-square)](https://github.com/sybila/biodivine-lib-sbml/actions?query=workflow%3Abuild)
[![Codecov](https://img.shields.io/codecov/c/github/sybila/biodivine-lib-sbml?style=flat-square)](https://codecov.io/gh/sybila/biodivine-lib-sbml)
[![GitHub issues](https://img.shields.io/github/issues/sybila/biodivine-lib-sbml?style=flat-square)](https://github.com/sybila/biodivine-lib-sbml/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-lib-sbml?style=flat-square)](https://github.com/sybila/biodivine-lib-sbml/commits/master)
[![Crates.io](https://img.shields.io/crates/l/biodivine-lib-sbml?style=flat-square)](https://github.com/sybila/biodivine-lib-sbml/blob/master/LICENSE)

# Biodivine/LibSBML

This crate provides a Rust interface for reading, editing, and validating Systems Biology Markup Language (SBML) files. Main features:

 - [x] Complete support for the SBML Level 3 Version 2 core specification.
 - [x] Validation of the *required* SBML conformance rules, including validation of proper namespace usage.
 - [x] Ability to (safely) edit invalid or partially corrupted files (e.g. to fix errors).
 - [x] Full access to the underlying XML document through the [`biodivine-lib-xml-dom`](https://github.com/sybila/biodivine-lib-xml-dom) interface.
 - [x] `Annotation`, `Notes` and other custom XML/HTML elements are fully accessible as raw `XmlElement` objects.
 - [x] Unofficial or unsupported features can be accessed using `DynamicProperty` or `DynamicChild` wrappers.

Reading a document preserves its text: the tree is a verbatim view of the source, so a read, edit
and write cycle keeps the content of `notes`, `annotation` and MathML elements intact. Where SBML
*interprets* text as a value (an identifier, a number or an enumerated value, in an attribute or in
a MathML element such as `ci`), the incidental whitespace around that value is ignored, because the
relevant XML Schema datatypes collapse it.

## Usage

You can import `lib-sbml` as any other Rust dependency:

**WARNING: `lib-sbml` is not published on crates.io yet. Instead, you can use the git dependency mechanism described below.**

```
[dependencies]
biodivine-lib-sbml = "0.0.1" 
# Or use git directly to include any unreleased changes:
biodivine-lib-sbml = { git = "https://github.com/sybila/biodivine-lib-sbml" }
```

Then, you can use `Sbml`, `Model` and other SBML wrapper objects that should directly map to the SBML specification:

```rust
use biodivine_lib_sbml::core::{SBase, SId, Species};
use biodivine_lib_sbml::xml::{
    OptionalDynamicChild, OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty,
    XmlChildDefault, XmlElement, XmlList, XmlWrapper,
};
use biodivine_lib_sbml::Sbml;

fn main() {
    let doc = Sbml::read_path("./test-inputs/model.sbml").expect("This document is not valid XML.");

    // First, we want to know if the document we
    // just read is valid SBML. Validating can be cancelled (see the
    // `cancel-this` crate), so it returns a `Result`; this example
    // registers no cancellation trigger.
    let issues = doc
        .validate()
        .expect("The validation cannot be cancelled here.");
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

    // For example, here, we are reading the list of qualitative species defined
    // in the sbml-qual package as a "generic" list of `XmlElement` objects.
    let qual_species: OptionalDynamicChild<XmlList<XmlElement>> =
        OptionalDynamicChild::new(model.xml_element(), "listOfQualitativeSpecies");
    println!(
        "This model has {} qualitative species.",
        qual_species.get_or_create().len(),
    );

    // We can also modify the model.

    // First, create a new instance of a `Species` object.
    let species_id = SId::try_from("sp_1").unwrap();
    let compartment_id = compartments.get(0).id().get();
    let s = Species::new(model.xml_element().document(), &species_id, &compartment_id);
    let species_name = "MySpecies".to_string();
    s.name().set_some(&species_name);

    // Then, add it to the current list of species.
    species.push(s);
    assert_eq!(species.get(0).name().get(), Some(species_name));

    // Finally, we can print the model back as XML:
    let xml_string = doc.to_xml_string().expect("Encoding error.");

    println!("{} ... ", &xml_string[..200]);
}
```

The code above is also available in `examples/basic_example.rs`. Hence, you can run it using `cargo run --example basic_example`. 

You can learn more about the library API in the official documentation.

## The SBML specification and its authors

The object model this crate implements, the summaries of the validation rules, and the class diagrams
in this documentation are adapted from the SBML specification, in particular:

> **The Systems Biology Markup Language (SBML): Language Specification for Level 3 Version 2 Core**,
> Release 2, 29 March 2019.
>
> Michael Hucka (Chair), Frank T. Bergmann, Claudine Chaouiya, Andreas Dräger, Stefan Hoops,
> Sarah M. Keating, Matthias König, Nicolas Le Novère, Chris J. Myers, Brett G. Olivier, Sven Sahle,
> James C. Schaff, Rahuman Sheriff, Lucian P. Smith, Dagmar Waltemath, Darren J. Wilkinson and
> Fengkai Zhang.
>
> SBML is developed and maintained by the SBML Editors, <sbml-editors@sbml.org>, <https://sbml.org>.

The specification text, the rule identifiers and the figures are reused here with the permission of
the SBML Editors: the class diagrams in `docs-images/` are the specification's own SVG sources, and
the one figure that had to be redrawn says so in `docs-images/README.md`, next to the source of every
other figure. Where this documentation quotes a rule, the rule identifier is the specification's, and
the wording is a summary rather than a copy.

## Roadmap

Future roadmap, including estimated development priorities:

 - [ ] **[high]** Python bindings using `pyo3` (the `sys` layer is under development).
 - [ ] **[medium]** WASM/JS bindings using `wasm-bindgen`.
 - [ ] **[medium]** Validation of the *recommendation* SBML conformance rules.
 - [ ] **[low]** Validation of the *good practice* SBML conformance rules.
 - [ ] **[low]** Support for SBML Level 1 (documents are read and named by the `unmodelled-level` lint).
 - [ ] **[low]** Automated Level 1 => Level 2 => Level 3 migrations.
 - [x] **[high]** SBML Level 2 (Versions 1-5) and Level 3 Version 1, as deltas on the Level 3 Version 2
   model (see `documentation::design`).
 - [x] **[high]** Idiomatic MathML support for the subset SBML uses (`math::MathNode`).
 - [x] **[medium]** Hooks for incorporating packages into the validation flow (`documentation::design`
   and the `extension` module: the cast and the injected rule set).
 - [ ] Support for SBML packages.
   * [ ] **[low]** Arrays
   * [ ] **[medium]** Hierarchical model composition
   * [ ] **[medium]** Distributions
   * [ ] **[low]** Dynamic structures
   * [x] **[medium]** Flux balance constraints
   * [ ] **[low]** Groups
   * [x] **[high]** Layout
   * [ ] **[high]** Rendering
   * [ ] **[medium]** Extended MathML
   * [ ] **[high]** Multi-state species
   * [x] **[high]** Qualitative models
   * [ ] **[low]** Spatial processes
