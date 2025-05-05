use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
#[cfg(feature = "python")]
use sbml_macros::PythonXmlChild;
use sbml_macros::{SBase, XmlWrapper};

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "python", derive(PythonXmlChild))]
pub struct Parameter(XmlElement);

impl Parameter {
    pub fn new(document: XmlDocument, id: &SId, constant: bool) -> Self {
        let obj = Parameter::new_empty(document, "parameter");
        obj.id().set(id);
        obj.constant().set(&constant);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalSbmlProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("constant")
    }
}
