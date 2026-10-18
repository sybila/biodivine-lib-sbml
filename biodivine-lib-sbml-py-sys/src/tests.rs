//! Tests of the wrapper layer, run inside an embedded interpreter.
//!
//! The crate's Python-facing behaviour is testable from Rust because `pyo3`'s `auto-initialize`
//! feature starts an interpreter for the test binary: the tests below build the module the
//! aggregate extension would build, call it from Python code, and check what came back. Behaviour
//! that only exists in the pure-Python layer (`biodivine.sbml`) is tested in the `biodivine-py`
//! repository instead.

use cancel_this::CancelAtomic;
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Builds the `biodivine_sys.sbml` submodule in a fresh interpreter state.
fn module(py: Python<'_>) -> Bound<'_, PyModule> {
    let module = PyModule::new(py, "biodivine_sys.sbml").expect("the module can be created");
    crate::register(&module).expect("register fills the module");
    module
}

#[test]
fn register_fills_the_module_with_every_documented_name() {
    Python::attach(|py| {
        let module = module(py);
        for name in crate::all_names() {
            assert!(
                module
                    .hasattr(name.to_string())
                    .expect("attribute lookup works"),
                "the module is missing `{name}`"
            );
        }
        // The version is the library's, not this crate's: a consumer reports one number for the
        // library it wraps, and the two manifests are kept in step by a test below.
        let version: String = module
            .getattr("__version__")
            .unwrap()
            .extract()
            .expect("__version__ is a string");
        assert_eq!(version, biodivine_lib_sbml::VERSION);
    });
}

#[test]
fn a_document_round_trips_through_the_bindings() {
    Python::attach(|py| {
        let module = module(py);
        let source = r#"<?xml version="1.0" encoding="UTF-8"?>
            <sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
                <model><listOfCompartments><compartment id="c" constant="true" size="1"/></listOfCompartments></model>
            </sbml>"#;
        let document = module
            .getattr("Sbml")
            .unwrap()
            .call_method1("read_str", (source,))
            .expect("the document parses");
        let level: Option<u32> = document
            .getattr("level")
            .unwrap()
            .extract()
            .expect("level is an int");
        assert_eq!(level, Some(3));
        let issues = document.call_method0("validate").expect("validation runs");
        let contents: Vec<String> = issues
            .try_iter()
            .expect("the report is iterable")
            .map(|issue| issue.expect("an issue").repr().expect("repr").to_string())
            .collect();
        assert!(
            contents.is_empty(),
            "a clean document reports no issues, found {contents:?}"
        );
        let written: String = document
            .call_method0("to_xml_string")
            .unwrap()
            .extract()
            .unwrap();
        assert!(written.contains(r#"level="3""#));
        assert!(written.contains("<compartment"), "written: {written}");
    });
}

#[test]
fn a_malformed_document_raises_the_xml_error() {
    Python::attach(|py| {
        let module = module(py);
        let result = module
            .getattr("Sbml")
            .unwrap()
            .call_method1("read_str", ("<sbml><unclosed></sbml>",));
        let error = result.expect_err("the document does not parse");
        let error_class = module.getattr("SbmlXmlError").unwrap();
        assert!(
            error.is_instance(py, &error_class),
            "the error is a SbmlXmlError, not a plain exception: {error}"
        );
    });
}

#[test]
fn a_document_can_be_edited_through_the_object_model() {
    Python::attach(|py| {
        let module = module(py);
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
            <model><listOfCompartments><compartment id="c" constant="true"/></listOfCompartments></model>
        </sbml>"#;
        let document = module
            .getattr("Sbml")
            .unwrap()
            .call_method1("read_str", (source,))
            .unwrap();
        let model = document.getattr("model").unwrap();
        let document_element = document.getattr("root").unwrap();
        let xml_document = document_element.call_method0("document").unwrap();

        // A list child is created on demand, and an item is appended to it.
        let species_list = model.call_method0("create_species").unwrap();
        let species = module
            .getattr("Species")
            .unwrap()
            .call_method1("new_empty", (&xml_document,))
            .unwrap();
        species.setattr("id", "s").unwrap();
        species.setattr("compartment", "c").unwrap();
        species.setattr("has_only_substance_units", false).unwrap();
        species.setattr("constant", false).unwrap();
        species.setattr("boundary_condition", false).unwrap();
        species_list.call_method1("append", (&species,)).unwrap();
        assert_eq!(species_list.len().unwrap(), 1);

        // Reading it back goes through the typed classes, not through raw XML.
        let read_back = species_list.get_item(0).unwrap();
        assert_eq!(
            read_back
                .getattr("id")
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "s"
        );
        assert_eq!(
            read_back
                .getattr("compartment")
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "c"
        );

        let issues = document.call_method0("validate").unwrap();
        let contents: Vec<String> = issues
            .try_iter()
            .unwrap()
            .map(|issue| issue.unwrap().repr().unwrap().to_string())
            .collect();
        assert!(
            contents.is_empty(),
            "the edited document is valid: {contents:?}"
        );
        let written: String = document
            .call_method0("to_xml_string")
            .unwrap()
            .extract()
            .unwrap();
        assert!(written.contains("listOfSpecies"), "written: {written}");
        assert!(written.contains(r#"id="s""#), "written: {written}");
    });
}

#[test]
fn a_mathml_formula_is_built_and_read_back() {
    Python::attach(|py| {
        let module = module(py);
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
            <model><listOfCompartments><compartment id="c" constant="true"/></listOfCompartments></model>
        </sbml>"#;
        let document = module
            .getattr("Sbml")
            .unwrap()
            .call_method1("read_str", (source,))
            .unwrap();
        let xml_document = document
            .getattr("root")
            .unwrap()
            .call_method0("document")
            .unwrap();
        let math_node = module.getattr("MathNode").unwrap();
        let times = module
            .getattr("MathOperator")
            .unwrap()
            .getattr("TIMES")
            .unwrap();

        // `k * 2`, built from the typed helpers: a `<ci>` and an integer `<cn>` under an `<apply>`.
        let symbol = math_node.call_method1("ci", (&xml_document, "k")).unwrap();
        let number = math_node
            .call_method1("integer", (&xml_document, 2))
            .unwrap();
        let apply = math_node
            .call_method1(
                "apply",
                (
                    &xml_document,
                    &times,
                    pyo3::types::PyList::new(py, [&symbol, &number]).unwrap(),
                ),
            )
            .unwrap();

        // Attach it to the kinetic law of a new reaction.
        let model = document.getattr("model").unwrap();
        let reactions = model.call_method0("create_reactions").unwrap();
        let reaction = module
            .getattr("Reaction")
            .unwrap()
            .call_method1("new_empty", (&xml_document,))
            .unwrap();
        reaction.setattr("id", "r").unwrap();
        reaction.setattr("reversible", false).unwrap();
        reactions.call_method1("append", (&reaction,)).unwrap();
        let law = reaction.call_method0("create_kinetic_law").unwrap();
        let math = law.call_method0("create_math").unwrap();
        math.call_method1("set_expression", (&apply,)).unwrap();

        // Reading it back gives the same typed tree.
        let expression = math.getattr("expression").unwrap();
        assert_eq!(
            expression
                .getattr("kind")
                .unwrap()
                .str()
                .unwrap()
                .to_string(),
            "apply"
        );
        let children = expression.getattr("children").unwrap();
        assert_eq!(children.len().unwrap(), 3, "operator and two arguments");
        assert_eq!(
            children
                .get_item(1)
                .unwrap()
                .getattr("kind")
                .unwrap()
                .str()
                .unwrap()
                .to_string(),
            "ci"
        );
        let written: String = document
            .call_method0("to_xml_string")
            .unwrap()
            .extract()
            .unwrap();
        assert!(written.contains("<times/>"), "written: {written}");
        assert!(written.contains("<ci>k</ci>"), "written: {written}");
    });
}

#[test]
fn a_cancelled_operation_reports_cancellation() {
    Python::attach(|py| {
        let module = module(py);
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2"><model/></sbml>"#;
        // A trigger registered on this thread aborts the next long operation. The bindings do not
        // expose the trigger (a Python caller interrupts with a signal), so this test exercises the
        // Rust-side mechanism the Python path uses.
        let trigger = CancelAtomic::new();
        trigger.cancel();
        // The parse consults the triggers registered on the calling thread, so a cancelled trigger
        // turns it into the library's cancellation error ...
        let outcome = cancel_this::on_atomic(trigger.clone(), || {
            biodivine_lib_sbml::Sbml::read_str(source)
        });
        let cancelled = outcome.expect_err("a cancelled operation must not succeed");
        assert!(matches!(
            cancelled,
            biodivine_lib_sbml::SbmlError::Cancelled(_)
        ));
        // ... which the wrapper maps onto the Python exception hierarchy: this is what a Python
        // caller sees when a signal interrupts the call.
        let error = crate::error::to_py_err(cancelled);
        let error_class = module.getattr("SbmlCancelledError").unwrap();
        assert!(error.is_instance(py, &error_class), "error: {error}");
    });
}

#[test]
fn an_unmodelled_level_is_reported_with_a_lint() {
    Python::attach(|py| {
        let module = module(py);
        // Level 1 is published (so the namespace is known) but its object model is not modelled,
        // which must be *said* rather than silently reported as a clean document.
        let source = r#"<sbml xmlns="http://www.sbml.org/sbml/level1" level="1" version="2"><model/></sbml>"#;
        let document = module
            .getattr("Sbml")
            .unwrap()
            .call_method1("read_str", (source,))
            .unwrap();
        let issues = document.call_method0("validate").unwrap();
        let first = issues.get_item(0).unwrap();
        let rule: String = first.getattr("rule").unwrap().extract().unwrap();
        assert_eq!(rule, "unmodelled-level");
    });
}
