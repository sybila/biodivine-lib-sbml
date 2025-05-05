use pyo3::{PyObject, PyResult, Python};

/// The role of [PythonPropertyConverter] is to convert between string-based representation in XML
/// and a Python type. Since we can't use generics, the return type is an arbitrary [PyObject].
///
/// Furthermore, note that in most instances, the methods could be left as static, but this
/// would mean we can't provide a boxed dynamic "converter" to avoid using generics in the
/// Python-accessible XML properties.
pub trait PythonPropertyConverter {
    /// Convert an XML attribute value into a Python object.
    fn try_from_string(&self, value: String, py: Python) -> PyResult<PyObject>;

    /// Convert a Python object into an XML attribute value.
    fn try_into_string(&self, value: PyObject, py: Python) -> PyResult<String>;
}

/// Types that implement [PythonPropertyType] provide a default implementation of
/// the [PythonPropertyConverter] which can be used to create
/// the [crate::xml::py::SbmlPropertyPy].
///
/// This trait can be automatically derived for our own `pyclass` types, or implemented
/// for other types that implement [pyo3::conversion::IntoPyObject]
/// and [pyo3::conversion::FromPyObject].
pub trait PythonPropertyType {
    fn converter() -> Box<dyn PythonPropertyConverter + Send + Sync>;
}
