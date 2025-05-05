use crate::core::SId;
use crate::xml::py::runtime_error;
use pyo3::{pymethods, PyResult};

#[pymethods]
impl SId {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Self::try_from(value.to_string()).map_err(runtime_error)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("SId({})", self.as_str()))
    }
}
