use crate::xml::py::{PythonXmlChild, PythonXmlChildConverter};
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use pyo3::{
    pyclass, pyfunction, pymethods, Bound, IntoPy, IntoPyObject, IntoPyObjectExt, Py, PyErr,
    PyObject, PyResult, Python,
};
use pyo3_stub_gen::{PyStubType, TypeInfo};
use pyo3_stub_gen_derive::gen_stub_pyfunction;

#[pyclass]
pub struct XmlListPy {
    generic_list: XmlList<XmlElement>,
    converter: Box<dyn PythonXmlChildConverter + Send + Sync>,
}

// TODO:
//  This is trying to add support for "generic-like" types, but it is not working.
//  The reason is that the type info is not separated for declaration site and "usage" site.
//  Ideally, supporting generic types should just be implemented inside TypeInfo.

impl Clone for XmlListPy {
    fn clone(&self) -> Self {
        XmlListPy {
            generic_list: self.generic_list.clone(),
            converter: self.converter.clone_self(),
        }
    }
}

impl PyStubType for XmlListPy {
    fn type_output() -> TypeInfo {
        TypeInfo::unqualified("XmlListPy[T]")
    }
}

// Note: The custom type works, but only if it is used in some other signature. It is not
// included on its own. Also, note that the only way to have a custom type is using a
// static trait, so there is no way for us to nicely incorporate the type name into the
// signature dynamically.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn test_function<'py>(_py: Python<'py>, list: Bound<'py, XmlListPy>) -> PyResult<ListWrapper> {
    unimplemented!();
}

pub struct ListWrapper(XmlListPy);

impl<'py> IntoPyObject<'py> for ListWrapper {
    type Target = XmlListPy;
    type Output = pyo3::Bound<'py, XmlListPy>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let p = Py::new(py, self.0)?;
        Ok(p.bind(py).clone())
    }
}

impl PyStubType for ListWrapper {
    fn type_output() -> TypeInfo {
        let t = format!("{}[XmlElement]", XmlListPy::type_output());
        TypeInfo::unqualified(t.as_str())
    }
}

impl<C: XmlWrapper + PythonXmlChild> PythonXmlChild for XmlList<C> {
    fn converter_unsafe() -> Box<dyn PythonXmlChildConverter + Send + Sync> {
        struct Internal {
            inner_converter: Box<dyn PythonXmlChildConverter + Send + Sync>,
        }

        impl Clone for Internal {
            fn clone(&self) -> Self {
                Internal {
                    inner_converter: self.inner_converter.clone_self(),
                }
            }
        }

        impl PythonXmlChildConverter for Internal {
            fn try_into_typed_child(&self, value: XmlElement, py: Python) -> PyResult<PyObject> {
                let generic_list: XmlList<XmlElement> = unsafe { XmlList::unchecked_cast(value) };
                let py_list = XmlListPy {
                    generic_list,
                    converter: self.inner_converter.clone_self(),
                };
                py_list.into_py_any(py)
            }

            fn try_from_typed_child(&self, value: PyObject, py: Python) -> PyResult<XmlElement> {
                let list = value.downcast_bound::<XmlListPy>(py)?;
                Ok(list.borrow().generic_list.xml_element().clone())
            }

            fn clone_self(&self) -> Box<dyn PythonXmlChildConverter + Send + Sync> {
                Box::new(self.clone())
            }
        }

        Box::new(Internal {
            inner_converter: C::converter_unsafe(),
        })
    }
}

#[pymethods]
impl XmlListPy {
    /// See [XmlList::get].
    pub fn get(&self, index: usize, py: Python) -> PyResult<PyObject> {
        let element = self.generic_list.get(index);
        self.converter.try_into_typed_child(element, py)
    }

    /// See [XmlList::insert].
    pub fn insert(&self, index: usize, value: PyObject, py: Python) -> PyResult<()> {
        let element = self.converter.try_from_typed_child(value, py)?;
        self.generic_list.insert(index, element);
        Ok(())
    }

    /// See [XmlList::remove].
    pub fn remove(&self, index: usize, py: Python) -> PyResult<PyObject> {
        let element = self.generic_list.remove(index);
        self.converter.try_into_typed_child(element, py)
    }

    /// See [XmlList::len].
    pub fn len(&self) -> usize {
        self.generic_list.len()
    }

    /// See [XmlList::is_empty].   
    pub fn is_empty(&self) -> bool {
        self.generic_list.is_empty()
    }
}
