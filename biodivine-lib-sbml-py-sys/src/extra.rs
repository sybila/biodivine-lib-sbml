//! The module-level surface: namespaces, the package predicates and the extension error.
//!
//! The library publishes data an SBML extension needs (the namespace prefixes and URLs, the
//! published namespace sets of each package) and a handful of free functions over them, plus the
//! error a package cast produces. This module binds that surface, and it is where a Python caller
//! finds the answers to "which URL does the FBC package use?" and "is this namespace a version of
//! Layout?".

use biodivine_lib_sbml::extension::ExtensionError;
use biodivine_lib_sbml::qual::get_outputs_from_transition as rust_outputs_from_transition;
use biodivine_lib_sbml::xml::XmlElement;
use biodivine_lib_sbml::{
    is_package_namespace_of as rust_is_package_namespace_of,
    package_of_namespace_url as rust_package_of_namespace_url,
    package_urls_of as rust_package_urls_of, written_url_of as rust_written_url_of,
    COMP_NAMESPACE_URLS, FBC_NAMESPACE_URLS, LAYOUT_NAMESPACE_URLS, NS_EMPTY, NS_FBC, NS_HTML,
    NS_LAYOUT, NS_MATHML, NS_QUAL, NS_SBML_CORE, NS_XSI, QUAL_NAMESPACE_URLS, SBML_ROOT_NAMESPACES,
    URL_EMPTY, URL_HTML, URL_MATHML, URL_PACKAGE_FBC, URL_PACKAGE_LAYOUT, URL_PACKAGE_QUAL,
    URL_SBML_CORE, URL_XSI,
};
use pyo3::prelude::*;

use crate::element::PyElement;
use crate::generated::qual::{PyQualOutput, PyTransition};

/// The error a cast of a core object to a package view produces.
///
/// The cast itself is Rust API (it is generic over the extension type), but its *failure* is data a
/// caller can inspect: which package the cast was for, and whether the document simply does not
/// declare it.
#[pyclass(
    name = "ExtensionError",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyExtensionError {
    pub(crate) inner: ExtensionError,
}

#[pymethods]
impl PyExtensionError {
    /// The namespace URL of the package the failing cast was for.
    #[getter]
    fn package(&self) -> &'static str {
        self.inner.package()
    }

    /// Whether the failure is that the document does not declare the package.
    ///
    /// A caller that is editing the document can then declare it and try again; a caller that is
    /// only reading cannot, because a document that does not declare a package does not say that
    /// it uses it.
    fn is_not_declared(&self) -> bool {
        self.inner.is_not_declared()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "ExtensionError(package={:?}, not_declared={})",
            self.inner.package(),
            self.inner.is_not_declared()
        )
    }
}

/// Whether `url` is an accepted namespace URL of the package whose conventional prefix is `prefix`.
#[pyfunction]
fn is_package_namespace_of(url: &str, prefix: &str) -> bool {
    rust_is_package_namespace_of(url, prefix)
}

/// The conventional prefix of the package that publishes `url`, or `None`.
#[pyfunction]
fn package_of_namespace_url(url: &str) -> Option<&'static str> {
    rust_package_of_namespace_url(url)
}

/// Every namespace URL of the package whose conventional prefix is `prefix`, or `None`.
#[pyfunction]
fn package_urls_of(prefix: &str) -> Option<Vec<&'static str>> {
    rust_package_urls_of(prefix).map(|urls| urls.to_vec())
}

/// The URL this library writes with the namespace whose conventional prefix is `prefix`, or `None`.
#[pyfunction]
fn written_url_of(prefix: &str) -> Option<&'static str> {
    rust_written_url_of(prefix)
}

/// Writes the `xsi:type` attribute of an element, declaring the XML Schema instance namespace on it
/// when necessary.
///
/// This is the write half of the `xsi:type` support: an SBML extension whose types are told apart by
/// `xsi:type` rather than by their local name calls it, and the declaration is introduced for it.
#[pyfunction]
fn set_xsi_type(element: &PyElement, value: &str) {
    biodivine_lib_sbml::set_xsi_type(&XmlElement::new(element.rust_element().clone()), value);
}

/// The outputs of a transition, in document order (an empty list when it declares none).
///
/// This is the free function of the Qual package that a reader of a qualitative model uses instead
/// of walking the transition's children itself.
#[pyfunction]
fn get_outputs_from_transition(transition: &PyTransition) -> Vec<PyQualOutput> {
    rust_outputs_from_transition(transition.inner.clone())
        .into_iter()
        .map(PyQualOutput::from)
        .collect()
}

/// Adds the module-level names: the namespace data, the package predicates and the error class.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyExtensionError>()?;

    // The namespaces, as `(prefix, url)` pairs.
    module.add("NS_SBML_CORE", NS_SBML_CORE)?;
    module.add("NS_MATHML", NS_MATHML)?;
    module.add("NS_XSI", NS_XSI)?;
    module.add("NS_HTML", NS_HTML)?;
    module.add("NS_EMPTY", NS_EMPTY)?;
    module.add("NS_FBC", NS_FBC)?;
    module.add("NS_LAYOUT", NS_LAYOUT)?;
    module.add("NS_QUAL", NS_QUAL)?;
    module.add("URL_SBML_CORE", URL_SBML_CORE)?;
    module.add("URL_MATHML", URL_MATHML)?;
    module.add("URL_XSI", URL_XSI)?;
    module.add("URL_HTML", URL_HTML)?;
    module.add("URL_EMPTY", URL_EMPTY)?;
    module.add("URL_PACKAGE_FBC", URL_PACKAGE_FBC)?;
    module.add("URL_PACKAGE_LAYOUT", URL_PACKAGE_LAYOUT)?;
    module.add("URL_PACKAGE_QUAL", URL_PACKAGE_QUAL)?;

    // The published namespace sets.
    module.add("SBML_ROOT_NAMESPACES", SBML_ROOT_NAMESPACES.to_vec())?;
    module.add("FBC_NAMESPACE_URLS", FBC_NAMESPACE_URLS.to_vec())?;
    module.add("LAYOUT_NAMESPACE_URLS", LAYOUT_NAMESPACE_URLS.to_vec())?;
    module.add("QUAL_NAMESPACE_URLS", QUAL_NAMESPACE_URLS.to_vec())?;
    module.add("COMP_NAMESPACE_URLS", COMP_NAMESPACE_URLS.to_vec())?;

    module.add_function(wrap_pyfunction!(is_package_namespace_of, module)?)?;
    module.add_function(wrap_pyfunction!(package_of_namespace_url, module)?)?;
    module.add_function(wrap_pyfunction!(package_urls_of, module)?)?;
    module.add_function(wrap_pyfunction!(written_url_of, module)?)?;
    module.add_function(wrap_pyfunction!(set_xsi_type, module)?)?;
    module.add_function(wrap_pyfunction!(get_outputs_from_transition, module)?)?;
    Ok(())
}
