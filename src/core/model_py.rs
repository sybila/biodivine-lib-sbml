use crate::core::{Model, SBase};
use crate::xml::py::{SbmlPropertyPy, XmlChildPy};
use pyo3::pymethods;

#[pymethods]
impl Model {
    #[pyo3(name = "id")]
    pub fn id_py(&self) -> SbmlPropertyPy {
        SbmlPropertyPy::new_optional(self.id())
    }

    #[pyo3(name = "substance_units")]
    pub fn substance_units_py(&self) -> SbmlPropertyPy {
        SbmlPropertyPy::new_optional(self.substance_units())
    }

    #[pyo3(name = "parameters")]
    pub fn parameters_py(&self) -> XmlChildPy {
        XmlChildPy::new_optional(self.parameters())
    }
}
