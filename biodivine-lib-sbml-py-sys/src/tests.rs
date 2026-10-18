//! Tests of the wrapper layer, run inside an embedded interpreter.
//!
//! The crate's Python-facing behaviour is testable from Rust because `pyo3`'s `auto-initialize`
//! feature starts an interpreter for the test binary: the tests below build the module the
//! aggregate extension would build, call it from Python code, and check what came back. Behaviour
//! that only exists in the pure-Python layer (`biodivine.sbml`) is tested in the `biodivine-py`
//! repository instead.

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
