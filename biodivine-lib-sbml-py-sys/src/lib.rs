//! The `_sys` layer: a thin PyO3 mirror of `biodivine-lib-sbml`.
//!
//! # Where this layer sits
//!
//! An SBML document reaches Python through four layers, and this crate is the second one:
//!
//! ```text
//! biodivine-lib-sbml          the Rust library (no PyO3 anywhere, by design)
//! biodivine-lib-sbml-py-sys   this crate: one Python class/function per Rust item, delegating
//!                             without adding behaviour
//! biodivine-sys               the native extension of the Biodivine Python bindings: links every
//!                             `_sys` crate into one module and calls [`register`] to fill its
//!                             `biodivine_sys.sbml` submodule
//! biodivine                   the pure-Python package; `biodivine.sbml` adds the Pythonic API
//! ```
//!
//! The last two layers live in the `biodivine-py` repository. This crate is not a Python extension
//! on its own: it is a library of `#[pyclass]` types that the aggregate extension links in. Because
//! every `_sys` crate ends up in the same native module, the SBML wrappers use the XML DOM wrappers
//! directly: an SBML object's `element` is xml-dom's `Element`, and a document is parsed with
//! xml-dom's `Document` underneath. Python therefore sees one `Element` class rather than one per
//! library, and passing an element from one library's API to the other's works.
//!
//! # The rules this layer follows
//!
//! * **[`BINDINGS.md`](https://github.com/sybila/biodivine-lib-sbml/blob/main/biodivine-lib-sbml-py-sys/BINDINGS.md)
//!   is the contract.** Every public item of the library appears there exactly once, marked bound
//!   (with the Python name) or not bound (with a reason), and the file is generated from rustdoc's
//!   output. The `bindings_manifest` test reads it and checks that every bound item exists in this
//!   crate's module, so the table cannot claim a capability the wrapper does not have.
//! * **No policy.** A binding does what the Rust method does, with the same name (in `snake_case`)
//!   and the same arguments. Two shapes differ, and `BINDINGS.md` names both: a typed *accessor*
//!   (which returns a borrowed view object in Rust) becomes a getter/setter pair on the owning
//!   class, and a generic function (which takes a Rust type parameter) becomes a method of the
//!   concrete class that supplies it.
//! * **No panics cross the boundary.** Rust has an ergonomic variant that panics and a `_checked`
//!   variant that returns a `Result`. Python has no panics, so this layer exposes the `_checked`
//!   semantics under the plain name: a value that does not parse raises `ValueError` instead of
//!   aborting the interpreter. The workspace sets no `panic = "abort"`, so even a bug would surface
//!   as `PanicException` rather than killing the process.
//! * **Long operations release the GIL and can be interrupted.** Reading, writing and validating a
//!   document run through the `interrupt` module, which is the XML DOM layer's
//!   implementation: a Python signal handler runs on the calling thread while the work runs on a
//!   worker thread under a `cancel-this` trigger.
//!
//! # GIL policy
//!
//! | operation | GIL | why |
//! | --- | --- | --- |
//! | `Sbml.read_str`, `read_path`, `write_path`, `to_xml_string` | released | parses or serializes a whole document |
//! | `Sbml.validate` | released | walks the whole document |
//! | everything else | held | one lock acquisition and a bounded read or write |
#![deny(rustdoc::broken_intra_doc_links, rustdoc::private_intra_doc_links)]

mod document;
mod element;
mod error;
mod extra;
mod generated;
mod interrupt;
mod list;
mod math;
mod validation;

#[cfg(test)]
mod bindings_manifest;
#[cfg(test)]
mod tests;

use pyo3::prelude::*;
use pyo3::types::PyList;

pub use crate::document::PySbml;
pub use crate::element::PyElement;
pub use crate::list::{PySbmlClass, PySbmlList};
pub use crate::math::{
    PyMath, PyMathArity, PyMathConstant, PyMathNode, PyMathNodeKind, PyMathOperator, PyMathSymbol,
    PySbmlUnits,
};
pub use crate::validation::{
    PyIssueSeverity, PyRuleDescriptor, PyRuleKind, PyRuleSet, PySbmlIssue,
};

/// Adds every class, exception and constant of the SBML bindings to `module`.
///
/// The aggregate extension (`biodivine-sys`) creates the `biodivine_sys.sbml` submodule and calls
/// this function on it; the module path is also the one every class of this crate declares, so the
/// classes and the module agree on where they live.
///
/// # Errors
///
/// Fails with the Python exception raised by the interpreter when an attribute cannot be added to
/// `module`.
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "__doc__",
        "Native bindings of biodivine-lib-sbml (the `_sys` layer).",
    )?;
    // The version of the library this crate wraps, read from the library's own manifest.
    module.add("__version__", biodivine_lib_sbml::VERSION)?;

    module.add_class::<PySbml>()?;
    module.add_class::<PySbmlIssue>()?;
    module.add_class::<PyRuleKind>()?;
    module.add_class::<PyIssueSeverity>()?;
    module.add_class::<PyRuleDescriptor>()?;
    module.add_class::<PyRuleSet>()?;

    error::register(module)?;
    validation::register(module)?;
    document::register(module)?;
    generated::register(module)?;
    math::register(module)?;
    // The names of the MathML enumerations belong to the classes the line above adds.
    generated::math_names::register(module)?;
    extra::register(module)?;

    module.add("__all__", PyList::new(module.py(), all_names())?)?;
    Ok(())
}

/// The names this module exports, in the order they are documented.
fn all_names() -> Vec<&'static str> {
    vec![
        "__version__",
        "Sbml",
        "SbmlIssue",
        "RuleKind",
        "SbmlIssueSeverity",
        "RuleDescriptor",
        "RuleSet",
        "SbmlError",
        "SbmlXmlError",
        "SbmlCancelledError",
        "SbmlDocumentError",
        "SbmlUnsupportedLevelError",
        "SbmlIoError",
        "SbmlValidationError",
    ]
}
