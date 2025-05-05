use crate::xml::py::{PythonXmlChild, PythonXmlChildConverter};
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use pyo3::{pyclass, pyfunction, pymethods, IntoPyObjectExt, PyObject, PyResult, Python};
use pyo3_stub_gen::{PyStubType, TypeInfo};
use pyo3_stub_gen_derive::gen_stub_pyfunction;

#[pyclass]
pub struct XmlListPy {
    generic_list: XmlList<XmlElement>,
    converter: Box<dyn PythonXmlChildConverter + Send + Sync>,
}

impl PyStubType for XmlListPy {
    fn type_output() -> TypeInfo {
        TypeInfo::unqualified("TestingCustomTypes | NotTestingCustomTypes")
    }
}

// Note: The custom type works, but only if it is used in some other signature. It is not
// included on its own. Also, note that the only way to have a custom type is using a
// static trait, so there is no way for us to nicely incorporate the type name into the
// signature dynamically.
#[gen_stub_pyfunction]
#[pyfunction]
pub fn test_function(_py: Python) -> PyResult<XmlListPy> {
    unimplemented!();
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
