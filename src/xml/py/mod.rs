mod xml_python_child;
mod xml_python_child_converter;
mod xml_python_property;
mod xml_python_property_converter;

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::{PyErr, PyErrArguments, PyResult};

pub use xml_python_child::*;
pub use xml_python_child_converter::*;
pub use xml_python_property::*;
pub use xml_python_property_converter::*;

/// Helper function to quickly throw a type error.
pub fn throw_type_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(PyTypeError::new_err(message))
}

/// Helper function to quickly throw a runtime error.
pub fn throw_runtime_error<T, A>(message: A) -> PyResult<T>
where
    A: Send + Sync + PyErrArguments + 'static,
{
    Err(runtime_error::<A>(message))
}

/// Helper function to quickly create a runtime error.
pub fn runtime_error<A>(message: A) -> PyErr
where
    A: Send + Sync + PyErrArguments + 'static,
{
    PyRuntimeError::new_err(message)
}
