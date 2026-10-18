//! [`PySbmlList`]: a typed list of child elements.
//!
//! In Rust a list is `XmlList<T>`, a generic container whose item type is a Rust type parameter. A
//! Python class cannot be generic over a Rust type parameter, so this module erases the parameter
//! behind a trait object: one `SbmlList` class serves every list, and the item type is recovered at
//! the boundary by the [`PySbmlClass`] implementation the wrapper classes get from their
//! declaration.
//!
//! The two directions are the reason the trait exists:
//!
//! * reading an item wraps the Rust child in its Python class (so `model.species[0]` is a
//!   `Species`, not an `Element`);
//! * writing an item unwraps the Python class back into the Rust wrapper, and a value of the wrong
//!   class is rejected with `TypeError` instead of being written into the document.

use std::sync::Arc;

use biodivine_lib_sbml::xml::{XmlElement, XmlList, XmlWrapper};
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::PyClass;

use crate::element::PyElement;

/// A wrapper class: a Python class that stands for one Rust wrapper type.
///
/// The implementation is generated together with the class (see `src/generated/`), so the two
/// cannot drift apart: the `Rust` associated type is the type the class wraps, and `wrap`/`unwrap`
/// are the only two conversions the rest of the binding performs.
pub trait PySbmlClass: PyClass<BaseType = pyo3::PyAny> + Clone + Send + 'static {
    /// The Rust wrapper type this class stands for.
    type Rust: XmlWrapper + Clone + Send + Sync + 'static;

    /// Wraps a Rust wrapper in this class.
    fn from_rust(item: Self::Rust) -> Self;

    /// Unwraps this class back into the Rust wrapper it holds.
    fn into_rust(self) -> Self::Rust;

    /// Reads this class out of a Python value, or fails with `TypeError`.
    ///
    /// The generated implementation is `value.extract::<Self>()`, which is why the method exists:
    /// the extraction bound is only satisfiable for the concrete class, not for a type parameter.
    fn extract_py(value: &Bound<'_, PyAny>) -> PyResult<Self>;
}

/// What a `SbmlList` can do, independent of the item type.
trait AnySbmlList: Send + Sync {
    /// The wrapper class of the items, for the `item_type` attribute and for error messages.
    fn item_class<'py>(&self, py: Python<'py>) -> Bound<'py, PyType>;
    /// The `listOf…` element that holds the items.
    fn element(&self) -> PyElement;
    /// The number of items.
    fn len(&self) -> usize;
    /// The item at `index`, or `None` when the index is out of range.
    fn get(&self, py: Python<'_>, index: usize) -> Option<Py<PyAny>>;
    /// Every item, in document order.
    fn items(&self, py: Python<'_>) -> Vec<Py<PyAny>>;
    /// Appends an item.
    fn push(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()>;
    /// Inserts an item at `index` (`index == len` appends).
    fn insert(&self, py: Python<'_>, index: usize, value: &Bound<'_, PyAny>) -> PyResult<()>;
    /// Removes and returns the item at `index`.
    fn remove(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyAny>>;
    /// Removes and returns the last item.
    fn pop(&self, py: Python<'_>) -> PyResult<Py<PyAny>>;
    /// Removes every item.
    fn clear(&self);
}

/// One erased list, for one item type.
struct TypedList<T: PySbmlClass> {
    inner: XmlList<T::Rust>,
}

impl<T: PySbmlClass> AnySbmlList for TypedList<T> {
    fn item_class<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        py.get_type::<T>()
    }

    fn element(&self) -> PyElement {
        PyElement::from(self.inner.xml_element().element().clone())
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn get(&self, py: Python<'_>, index: usize) -> Option<Py<PyAny>> {
        let item = self.inner.get_checked(index)?;
        Some(wrap::<T>(py, item))
    }

    fn items(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        (0..self.inner.len())
            .filter_map(|index| self.get(py, index))
            .collect()
    }

    fn push(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.push(unwrap::<T>(py, value)?);
        Ok(())
    }

    fn insert(&self, py: Python<'_>, index: usize, value: &Bound<'_, PyAny>) -> PyResult<()> {
        if index > self.inner.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "insert index {index} is out of range for a list of {} items",
                self.inner.len()
            )));
        }
        self.inner.insert(index, unwrap::<T>(py, value)?);
        Ok(())
    }

    fn remove(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyAny>> {
        if index >= self.inner.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "remove index {index} is out of range for a list of {} items",
                self.inner.len()
            )));
        }
        Ok(wrap::<T>(py, self.inner.remove(index)))
    }

    fn pop(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if self.inner.is_empty() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "pop from an empty list",
            ));
        }
        Ok(wrap::<T>(py, self.inner.pop()))
    }

    fn clear(&self) {
        while !self.inner.is_empty() {
            self.inner.remove(0);
        }
    }
}

/// Wraps a Rust item in its Python class.
fn wrap<T: PySbmlClass>(py: Python<'_>, item: T::Rust) -> Py<PyAny> {
    Py::new(py, T::from_rust(item))
        .expect("allocating a wrapper object cannot fail")
        .into_any()
}

/// Unwraps a Python item back into its Rust wrapper.
fn unwrap<T: PySbmlClass>(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<T::Rust> {
    let item = T::extract_py(value).map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(format!(
            "expected an object of class `{}`, got `{}`",
            py.get_type::<T>()
                .name()
                .map(|it| it.to_string())
                .unwrap_or_default(),
            value
                .get_type()
                .name()
                .map(|it| it.to_string())
                .unwrap_or_else(|_| "?".to_string())
        ))
    })?;
    Ok(item.into_rust())
}

/// A list of SBML child elements.
///
/// The list is a view of the document, not a copy: reading an item reads the element, and appending
/// or removing an item edits the document. Iteration, `len` and indexing behave like a Python
/// sequence, and every item is returned as its typed class.
#[pyclass(
    name = "SbmlList",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub struct PySbmlList {
    inner: Arc<dyn AnySbmlList>,
}

impl PySbmlList {
    /// Erases the item type of a Rust list.
    pub(crate) fn new<T: PySbmlClass>(inner: XmlList<T::Rust>) -> Self {
        PySbmlList {
            inner: Arc::new(TypedList::<T> { inner }),
        }
    }

    /// The Rust list of this object, for a constructor that takes one.
    ///
    /// A constructor such as `Curve::new(document, segments)` takes the list itself rather than
    /// items, so the binding hands it the list element behind this object.
    pub(crate) fn rust_list<T: XmlWrapper>(&self) -> XmlList<T> {
        XmlList::from(XmlElement::new(self.inner.element().rust_element().clone()))
    }
}

#[pymethods]
impl PySbmlList {
    /// The number of items.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// The item at `index`; a negative index counts from the end, as in any Python sequence.
    fn __getitem__(&self, py: Python<'_>, index: isize) -> PyResult<Py<PyAny>> {
        let len = isize::try_from(self.inner.len()).expect("a list length fits in an isize");
        let index = if index < 0 { len + index } else { index };
        if index < 0 || index >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "list index out of range",
            ));
        }
        let index = usize::try_from(index).expect("the index is non-negative");
        self.inner
            .get(py, index)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("list index out of range"))
    }

    /// Every item, in document order.
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let items = pyo3::types::PyList::new(py, self.inner.items(py))?;
        Ok(items.call_method0("__iter__")?.unbind())
    }

    /// Replaces the item at `index`.
    fn __setitem__(&self, py: Python<'_>, index: usize, value: &Bound<'_, PyAny>) -> PyResult<()> {
        if index >= self.inner.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "list assignment index out of range",
            ));
        }
        self.inner.remove(py, index)?;
        self.inner.insert(py, index, value)
    }

    /// Deletes the item at `index`.
    fn __delitem__(&self, py: Python<'_>, index: usize) -> PyResult<()> {
        self.inner.remove(py, index).map(|_| ())
    }

    /// Appends an item to the end of the list.
    fn append(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.push(py, value)
    }

    /// Inserts an item before `index`.
    fn insert(&self, py: Python<'_>, index: usize, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.insert(py, index, value)
    }

    /// Removes and returns the item at `index` (the last item when no index is given).
    #[pyo3(signature = (index = None))]
    fn pop(&self, py: Python<'_>, index: Option<isize>) -> PyResult<Py<PyAny>> {
        match index {
            None => self.inner.pop(py),
            Some(index) => {
                let len =
                    isize::try_from(self.inner.len()).expect("a list length fits in an isize");
                let index = if index < 0 { len + index } else { index };
                if index < 0 || index >= len {
                    return Err(pyo3::exceptions::PyIndexError::new_err(
                        "pop index out of range",
                    ));
                }
                self.inner.remove(
                    py,
                    usize::try_from(index).expect("the index is non-negative"),
                )
            }
        }
    }

    /// Removes every item.
    fn clear(&self) {
        self.inner.clear();
    }

    /// The class of the items of this list.
    #[getter]
    fn item_type<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        self.inner.item_class(py)
    }

    /// The element that holds the items (the `listOf…` element).
    #[getter]
    fn element(&self) -> PyElement {
        self.inner.element()
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let class = self.inner.item_class(py);
        let name = class.name().map(|it| it.to_string()).unwrap_or_default();
        Ok(format!("SbmlList[{name}]({} items)", self.inner.len()))
    }
}
