use crate::core::{BaseUnit, MetaId, Model, Parameter, SId, SboTerm};
use crate::xml::py::{runtime_error, SbmlPropertyPy, XmlChildPy, XmlListPy};
use crate::xml::XmlElement;
use crate::Sbml;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pymethods, pymodule, Bound, PyResult, Python};

#[pymodule]
fn biodivine_lib_sbml(_py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<XmlElement>()?;
    module.add_class::<SbmlPropertyPy>()?;
    module.add_class::<XmlChildPy>()?;
    module.add_class::<XmlListPy>()?;
    module.add_class::<Sbml>()?;
    module.add_class::<Model>()?;
    module.add_class::<SId>()?;
    module.add_class::<MetaId>()?;
    module.add_class::<SboTerm>()?;
    module.add_class::<BaseUnit>()?;
    module.add_class::<Parameter>()?;
    Ok(())
}

#[pymethods]
impl Sbml {
    #[staticmethod]
    #[pyo3(name = "read_path")]
    pub fn read_path_py(path: &str) -> PyResult<Sbml> {
        Sbml::read_path(path).map_err(runtime_error)
    }

    #[pyo3(name = "model")]
    pub fn model_py(&self) -> XmlChildPy {
        XmlChildPy::new_optional(self.model())
    }
}
