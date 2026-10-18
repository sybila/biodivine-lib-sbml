//! [`PySbml`]: the handle to one SBML document.
//!
//! The doc comments on `#[pymethods]` become Python docstrings, so they are written for Python
//! readers: plain text plus a `Raises:` section, without rustdoc links.

use biodivine_lib_sbml::xml::{OptionalXmlChild, XmlProperty};
use biodivine_lib_sbml::Sbml;
use biodivine_lib_xml_dom_sys::PyElement;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};

use crate::error::value_err;
use crate::interrupt::run_interruptible;
use crate::validation::PySbmlIssue;

/// An SBML document: the `<sbml>` container, its model and the validation report.
///
/// A document is a cheap, thread-safe handle (it is the XML DOM handle plus the `<sbml>` element),
/// so passing it to another thread or returning it from a function does not copy the tree.
///
/// Two things are worth knowing before using the classes below it:
///
/// * reading keeps the text of the document verbatim, so a read/edit/write cycle does not rewrite
///   `notes`, `annotation` or MathML content;
/// * the object model resolves for a document of a level and version this library models. A
///   document of another published level still parses, reports no model and says so with the
///   `unmodelled-level` lint, so "no issues" is never reported for a document nothing checked.
#[pyclass(name = "Sbml", module = "biodivine_sys.sbml", from_py_object)]
#[derive(Debug, Clone)]
pub struct PySbml {
    pub(crate) inner: Sbml,
}

impl From<Sbml> for PySbml {
    fn from(inner: Sbml) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PySbml {
    /// Parses a document from a string.
    ///
    /// Raises:
    ///     SbmlXmlError: if the string is not a well-formed XML 1.0 document with a single root
    ///         element, or if it declares an encoding other than UTF-8.
    ///     SbmlDocumentError: if the document has no root element.
    ///     BaseException: whatever a Python signal handler raises to interrupt a long parse.
    #[staticmethod]
    fn read_str(py: Python<'_>, source: &str) -> PyResult<PySbml> {
        let size = source.len();
        let source = source.to_string();
        let document = run_interruptible(py, size, move || Sbml::read_str(&source))?;
        Ok(document.into())
    }

    /// Reads and parses a document from a file.
    ///
    /// Raises:
    ///     SbmlXmlError: if the file is not a well-formed XML document.
    ///     SbmlDocumentError: if the document has no root element.
    ///     SbmlIoError: if the file cannot be read.
    #[staticmethod]
    fn read_path(py: Python<'_>, path: &str) -> PyResult<PySbml> {
        // The size of the file is the measure of the work; a missing or unreadable file is
        // reported by the read itself, with the same error it would have produced anyway.
        let size = std::fs::metadata(path)
            .map(|it| it.len() as usize)
            .unwrap_or(0);
        let path = path.to_string();
        let document = run_interruptible(py, size, move || Sbml::read_path(&path))?;
        Ok(document.into())
    }

    /// Serializes the document into a file.
    ///
    /// Raises:
    ///     SbmlIoError: if the file cannot be written.
    #[pyo3(signature = (path))]
    fn write_path(&self, py: Python<'_>, path: &str) -> PyResult<()> {
        let document = self.inner.clone();
        let path = path.to_string();
        run_interruptible(py, self.inner.root().document().node_count(), move || {
            document.write_path(&path)
        })
    }

    /// Serializes the document into a string.
    ///
    /// Raises:
    ///     SbmlXmlError: if the document cannot be serialized.
    fn to_xml_string(&self, py: Python<'_>) -> PyResult<String> {
        let document = self.inner.clone();
        run_interruptible(py, self.inner.root().document().node_count(), move || {
            document.to_xml_string()
        })
    }

    /// The `level` attribute of the `<sbml>` container, or `None` if it is missing or malformed.
    #[getter]
    fn level(&self) -> PyResult<Option<u32>> {
        self.inner.level().get_checked().map_err(value_err)
    }

    /// The `version` attribute of the `<sbml>` container, or `None` if it is missing or malformed.
    #[getter]
    fn version(&self) -> PyResult<Option<u32>> {
        self.inner.version().get_checked().map_err(value_err)
    }

    /// The `<sbml>` element itself: the escape hatch to the XML tree.
    #[getter]
    fn root(&self) -> PyElement {
        PyElement::from(self.inner.root().element().clone())
    }

    /// Validates the document against the rules of the SBML specification and returns the issues.
    ///
    /// An empty list means the document is valid. An issue of a specification rule has the id the
    /// specification gives that rule; an issue of a library lint has an id of its own (such as
    /// `not-validated`) and means the document is valid SBML but the library has something to say
    /// about it.
    ///
    /// A document whose model could not be resolved (a foreign root namespace, or a published level
    /// this library does not model) is *not* silently valid: it reports `20101` or
    /// `unmodelled-level`.
    ///
    /// Raises:
    ///     SbmlCancelledError: if a cancellation trigger fires.
    ///     BaseException: whatever a Python signal handler raises to interrupt the walk.
    fn validate(&self, py: Python<'_>) -> PyResult<Vec<PySbmlIssue>> {
        let document = self.inner.clone();
        let issues = run_interruptible(py, self.inner.root().document().node_count(), move || {
            document.validate()
        })?;
        Ok(issues
            .into_iter()
            .map(|inner| PySbmlIssue { inner })
            .collect())
    }

    /// The document a child element belongs to, or `None` when `element` is not inside an
    /// `<sbml>` document.
    #[staticmethod]
    fn try_for_child_element(element: &PyElement) -> Option<PySbml> {
        let element = biodivine_lib_sbml::xml::XmlElement::new(element.rust_element().clone());
        Sbml::try_for_child_element(&element).map(PySbml::from)
    }

    fn __repr__(&self) -> PyResult<String> {
        let level = self
            .level()?
            .map_or_else(|| "?".to_string(), |it| it.to_string());
        let version = self
            .version()?
            .map_or_else(|| "?".to_string(), |it| it.to_string());
        let model = if self.inner.model().get().is_some() {
            "model"
        } else {
            "no model"
        };
        Ok(format!("Sbml(level={level}, version={version}, {model})"))
    }
}

/// Registers the classes of this module (the class objects themselves are added by `lib.rs`).
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = (
        module,
        PyTuple::empty(module.py()),
        module.py().get_type::<PyType>(),
    );
    Ok(())
}
