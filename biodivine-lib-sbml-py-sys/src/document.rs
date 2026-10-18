//! [`PySbml`]: the handle to one SBML document.
//!
//! The doc comments on `#[pymethods]` become Python docstrings, so they are written for Python
//! readers: plain text plus a `Raises:` section, without rustdoc links.

use biodivine_lib_sbml::core::{MetaId, SBase, SId, SboTerm};
use biodivine_lib_sbml::xml::{OptionalXmlChild, SbmlObject, XmlChild, XmlElement, XmlProperty};
use biodivine_lib_sbml::Sbml;
use biodivine_lib_sbml::{
    xml::Namespace, NS_EMPTY, NS_FBC, NS_HTML, NS_LAYOUT, NS_MATHML, NS_QUAL, NS_SBML_CORE, NS_XSI,
};
use biodivine_lib_xml_dom_sys::PyElement;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};

use crate::error::value_err;
use crate::generated::core::PyModel;
use crate::interrupt::run_interruptible;
use crate::validation::PySbmlIssue;
use biodivine_lib_sbml::core::Model;

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

    /// The `model` child of the container, or `None` when the document has none.
    #[getter]
    fn model(&self) -> Option<PyModel> {
        self.inner.model().get().map(PyModel::from)
    }

    /// Creates the `model` child if it does not exist and returns it.
    fn create_model(&self) -> PyModel {
        let view = self.inner.model();
        match OptionalXmlChild::get(&view) {
            Some(existing) => PyModel::from(existing),
            None => {
                let created = Model::new_empty(self.inner.root().document());
                OptionalXmlChild::set(&view, created.clone());
                PyModel::from(created)
            }
        }
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

    /// The element of the container, which is the same as `root`.
    #[getter]
    fn element(&self) -> PyElement {
        PyElement::from(self.inner.root().element().clone())
    }

    /// The `id` attribute of the container, or `None` (the specification gives it no meaning).
    #[getter]
    fn id(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&<Sbml as SBase>::id(&self.inner))
            .map(|it| it.map(|it| it.as_str().to_string()))
            .map_err(value_err)
    }

    /// Sets the `id` attribute of the container.
    #[setter]
    fn set_id(&self, value: Option<&str>) -> PyResult<()> {
        let view = <Sbml as SBase>::id(&self.inner);
        match value {
            Some(value) => {
                let value = SId::try_from(value).map_err(value_err)?;
                XmlProperty::set_raw(&view, value.as_str().to_string()).map_err(value_err)?;
            }
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }
        Ok(())
    }

    /// The `name` attribute of the container, or `None`.
    #[getter]
    fn name(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&<Sbml as SBase>::name(&self.inner)).map_err(value_err)
    }

    /// Sets the `name` attribute of the container.
    #[setter]
    fn set_name(&self, value: Option<&str>) -> PyResult<()> {
        let view = <Sbml as SBase>::name(&self.inner);
        match value {
            Some(value) => XmlProperty::set_raw(&view, value.to_string()).map_err(value_err)?,
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }
        Ok(())
    }

    /// The `metaid` attribute of the container, or `None`.
    #[getter]
    fn meta_id(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&<Sbml as SBase>::meta_id(&self.inner))
            .map(|it| it.map(|it| it.as_str().to_string()))
            .map_err(value_err)
    }

    /// Sets the `metaid` attribute of the container.
    #[setter]
    fn set_meta_id(&self, value: Option<&str>) -> PyResult<()> {
        let view = <Sbml as SBase>::meta_id(&self.inner);
        match value {
            Some(value) => {
                let value = MetaId::try_from(value).map_err(value_err)?;
                XmlProperty::set_raw(&view, value.as_str().to_string()).map_err(value_err)?;
            }
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }
        Ok(())
    }

    /// The `sboTerm` attribute of the container, or `None`.
    #[getter]
    fn sbo_term(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&<Sbml as SBase>::sbo_term(&self.inner))
            .map(|it| it.map(|it| it.as_str().to_string()))
            .map_err(value_err)
    }

    /// Sets the `sboTerm` attribute of the container.
    #[setter]
    fn set_sbo_term(&self, value: Option<&str>) -> PyResult<()> {
        let view = <Sbml as SBase>::sbo_term(&self.inner);
        match value {
            Some(value) => {
                let value = SboTerm::try_from(value).map_err(value_err)?;
                XmlProperty::set_raw(&view, value.as_str().to_string()).map_err(value_err)?;
            }
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }
        Ok(())
    }

    /// The `notes` child element of the container, or `None`.
    #[getter]
    fn notes(&self) -> Option<PyElement> {
        XmlChild::get_raw(&<Sbml as SBase>::notes(&self.inner))
            .map(|it| PyElement::from(it.element().clone()))
    }

    /// The `annotation` child element of the container, or `None`.
    #[getter]
    fn annotation(&self) -> Option<PyElement> {
        XmlChild::get_raw(&<Sbml as SBase>::annotation(&self.inner))
            .map(|it| PyElement::from(it.element().clone()))
    }

    /// The container of this container, which is itself.
    #[getter]
    fn sbml_root(&self) -> PySbml {
        self.clone()
    }

    /// The local name of the element the container is: `sbml`.
    #[classattr]
    const ELEMENT_NAME: &'static str = <Sbml as SbmlObject>::ELEMENT_NAME;

    /// The namespace the container is written in, as a `(prefix, url)` pair.
    #[classattr]
    const ELEMENT_NAMESPACE: (&'static str, &'static str) = <Sbml as SbmlObject>::ELEMENT_NAMESPACE;

    /// The `xsi:type` value that tells the container apart from a sibling, or `None`.
    #[classattr]
    const XSI_TYPE: Option<&'static str> = <Sbml as SbmlObject>::XSI_TYPE;

    /// Every local name this type recognizes.
    #[staticmethod]
    fn element_names() -> Vec<&'static str> {
        <Sbml as SbmlObject>::element_names().to_vec()
    }

    /// Whether `element` is an `<sbml>` element of a published SBML namespace.
    #[staticmethod]
    fn matches_element(element: &PyElement) -> bool {
        <Sbml as SbmlObject>::matches_element(&XmlElement::new(element.rust_element().clone()))
    }

    /// Whether this container is a valid `<sbml>` element.
    fn is_element(&self) -> bool {
        <Sbml as SbmlObject>::is_element(&self.inner)
    }

    /// The container of `element`, or `None` when it is not an `<sbml>` element.
    #[staticmethod]
    fn try_from_element(element: &PyElement) -> Option<PySbml> {
        <Sbml as SbmlObject>::try_from_element(XmlElement::new(element.rust_element().clone()))
            .map(PySbml::from)
    }

    /// Declares an SBML package in this document, so that its elements and attributes can be used.
    ///
    /// The package is given as a `(prefix, url)` pair, which is what the namespace constants of the
    /// module are. Declaring a package that the document already declares does nothing.
    ///
    /// Raises:
    ///     ValueError: if the namespace cannot be declared in this document.
    #[pyo3(signature = (prefix, url))]
    fn activate_package(&self, prefix: &str, url: &str) -> PyResult<()> {
        let namespace = known_namespace(prefix, url)
            .ok_or_else(|| value_err(format!("unknown SBML namespace {prefix:?} -> {url:?}")))?;
        self.inner.activate_package(namespace).map_err(value_err)
    }

    /// The prefix with which `url` is declared in this document, or `None` when it is not declared.
    ///
    /// Raises:
    ///     ValueError: if the namespace cannot be resolved in this document.
    #[pyo3(signature = (prefix, url))]
    fn declared_package_prefix(&self, prefix: &str, url: &str) -> PyResult<Option<String>> {
        let namespace = known_namespace(prefix, url)
            .ok_or_else(|| value_err(format!("unknown SBML namespace {prefix:?} -> {url:?}")))?;
        self.inner
            .declared_package_prefix(namespace)
            .map(Some)
            .map_err(value_err)
    }

    /// Whether this document declares the package `(prefix, url)` in a version this library models.
    #[pyo3(signature = (prefix, url))]
    fn declares_modelled_package(&self, prefix: &str, url: &str) -> bool {
        match known_namespace(prefix, url) {
            Some(namespace) => self.inner.declares_modelled_package(namespace),
            None => false,
        }
    }

    /// The document a child object belongs to.
    #[staticmethod]
    fn try_for_child(child: &Bound<'_, PyAny>) -> PyResult<Option<PySbml>> {
        let element = child.getattr("element")?;
        let element: PyElement = element.extract()?;
        Ok(PySbml::try_for_child_element(&element))
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

/// The namespaces this library writes with, as the `(prefix, url)` pairs the Python module
/// publishes. A namespace argument is looked up here, because the Rust API takes a
/// `(&'static str, &'static str)` pair and a Python string is not one.
const KNOWN_PREFIXES: [Namespace; 8] = [
    NS_SBML_CORE,
    NS_MATHML,
    NS_XSI,
    NS_HTML,
    NS_EMPTY,
    NS_FBC,
    NS_LAYOUT,
    NS_QUAL,
];

/// The namespace a `(prefix, url)` pair names, or `None` when this library does not know it.
fn known_namespace(prefix: &str, url: &str) -> Option<Namespace> {
    KNOWN_PREFIXES
        .iter()
        .copied()
        .find(|(known_prefix, known_url)| *known_prefix == prefix && *known_url == url)
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
