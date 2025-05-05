use crate::core::Parameter;
use crate::xml::py::SbmlPropertyPy;
use pyo3::pymethods;

#[pymethods]
impl Parameter {
    #[pyo3(name = "id")]
    pub fn id_py(&self) -> SbmlPropertyPy {
        SbmlPropertyPy::new_required(self.id())
    }
}
