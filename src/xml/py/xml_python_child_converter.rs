use crate::xml::XmlElement;
use pyo3::{PyObject, PyResult, Python};

pub trait PythonXmlChildConverter {
    fn try_into_typed_child(&self, value: XmlElement, py: Python) -> PyResult<PyObject>;
    fn try_from_typed_child(&self, value: PyObject, py: Python) -> PyResult<XmlElement>;
    fn clone_self(&self) -> Box<dyn PythonXmlChildConverter + Send + Sync>;
}

pub trait PythonXmlChild {
    fn converter_unsafe() -> Box<dyn PythonXmlChildConverter + Send + Sync>;
}
