use crate::xml::{Child, XmlElement, XmlWrapper};
use pyo3::{PyObject, PyResult, Python};

pub trait PythonXmlChildConverter {
    fn try_into_typed_child(&self, value: XmlElement, py: Python) -> PyResult<PyObject>;
    fn try_from_typed_child(&self, value: PyObject, py: Python) -> PyResult<XmlElement>;
}

pub trait PythonXmlChild {
    fn converter<T: XmlWrapper>(child: Child<T>) -> Box<dyn PythonXmlChildConverter + Send + Sync>;
}
