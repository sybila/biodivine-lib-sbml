use crate::xml::py::throw_type_error;
use crate::xml::py::{PythonXmlChild, PythonXmlChildConverter};
use crate::xml::{Child, OptionalChild, OptionalXmlChild, XmlChild, XmlElement, XmlWrapper};
use pyo3::{pyclass, pymethods, PyObject, PyResult, Python};

#[pyclass]
pub struct XmlChildPy {
    parent: XmlElement,
    name: &'static str,
    namespace_url: &'static str,
    is_required: bool,
    converter: Box<dyn PythonXmlChildConverter + Send + Sync>,
}

impl XmlChildPy {
    pub fn new_optional<T: XmlWrapper + PythonXmlChild>(child: OptionalChild<T>) -> XmlChildPy {
        XmlChildPy {
            parent: child.0.parent.clone(),
            name: child.0.name,
            namespace_url: child.0.namespace_url,
            is_required: false,
            converter: T::converter(child.0),
        }
    }

    fn as_generic_child(&self) -> Child<XmlElement> {
        Child::new(&self.parent, self.name, self.namespace_url)
    }

    fn as_optional_generic_child(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(&self.parent, self.name, self.namespace_url)
    }
}

#[pymethods]
impl XmlChildPy {
    /// See [XmlChild::parent].
    pub fn parent(&self) -> XmlElement {
        self.parent.clone()
    }

    /// See [XmlChild::name].
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    /// See [XmlChild::namespace_url].   
    pub fn namespace_url(&self) -> String {
        self.namespace_url.to_string()
    }

    /// See [XmlChild::get_raw].
    pub fn get_raw(&self) -> Option<XmlElement> {
        self.as_generic_child().get_raw()
    }

    /// See [XmlChild::set_raw].
    pub fn set_raw(&self, value: XmlElement) {
        self.as_generic_child().set_raw(value);
    }

    /// See [XmlChild::clear_raw].
    pub fn clear_raw(&self) -> Option<XmlElement> {
        self.as_generic_child().clear_raw()
    }

    /// See [crate::xml::OptionalXmlChild::get] and [crate::xml::RequiredXmlChild::get].
    ///
    /// The method acts as either required or optional, depending on the configuration of the
    /// underlying object.
    pub fn get(&self, py: Python) -> PyResult<PyObject> {
        match self.get_raw() {
            None => {
                if self.is_required {
                    throw_type_error(format!(
                        "Child element `{}` is required and cannot be `None`.",
                        self.name
                    ))
                } else {
                    Ok(py.None())
                }
            }
            Some(element) => self.converter.try_into_typed_child(element, py),
        }
    }

    /// See [crate::xml::OptionalXmlChild::set] and [crate::xml::RequiredXmlChild::set].
    ///
    /// The method acts as either required or optional, depending on the configuration of the
    /// underlying object.
    pub fn set(&self, value: PyObject, py: Python) -> PyResult<PyObject> {
        if value.is_none(py) {
            if self.is_required {
                throw_type_error(format!(
                    "Child element `{}` is required and cannot be `None`.",
                    self.name
                ))
            } else {
                self.clear(py)
            }
        } else {
            let element = self.converter.try_from_typed_child(value, py)?;
            let previous = self.as_generic_child().set_raw(element);
            match previous {
                None => Ok(py.None()),
                Some(previous) => self.converter.try_into_typed_child(previous, py),
            }
        }
    }

    /// See [crate::xml::OptionalXmlChild::is_set].
    ///
    /// For valid documents and required children, this method should always return `true`.
    /// However, if the document contains an error, this method can return `false` even if the
    /// child is required.
    pub fn is_set(&self) -> bool {
        self.as_optional_generic_child().is_set()
    }

    /// See [crate::xml::OptionalXmlChild::clear].
    ///
    /// This function is only valid for optional child elements. For required elements,
    /// it throws a runtime error (you can still use `clear_raw` to remove the child without
    /// enforcing validation rules).
    pub fn clear(&self, py: Python) -> PyResult<PyObject> {
        match self.clear_raw() {
            None => Ok(py.None()),
            Some(element) => self.converter.try_into_typed_child(element, py),
        }
    }
}
