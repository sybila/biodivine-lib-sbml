use crate::constants::namespaces::Namespace;
use crate::xml::py::{
    runtime_error, throw_runtime_error, throw_type_error, PythonPropertyConverter,
    PythonPropertyType,
};
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, SbmlProperty, XmlElement, XmlProperty,
    XmlPropertyType,
};
use pyo3::{pyclass, pymethods, PyObject, PyResult, Python};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass]
pub struct SbmlPropertyPy {
    // XML element for which the property is relevant.
    element: XmlElement,
    // The name of the XML attribute.
    name: &'static str,
    // The expected namespace of the XML attribute.
    property_namespace: Namespace,
    // The expected namespace of the XML element.
    element_namespace: Namespace,
    // A converter which translates between XML attribute values and Python objects.
    converter: Box<dyn PythonPropertyConverter + Send + Sync>,
    // Indicates that the value of this property is required, meaning the getters and setters
    // must not accept `None` as a valid value.
    is_required: bool,
}

impl SbmlPropertyPy {
    pub fn new_optional<T: XmlPropertyType + PythonPropertyType>(
        property: OptionalSbmlProperty<T>,
    ) -> SbmlPropertyPy {
        SbmlPropertyPy {
            element: property.0.element.clone(),
            name: property.0.name,
            property_namespace: property.0.property_namespace,
            element_namespace: property.0.element_namespace,
            is_required: false,
            converter: T::converter(),
        }
    }

    pub fn new_required<T: XmlPropertyType + PythonPropertyType>(
        property: RequiredSbmlProperty<T>,
    ) -> SbmlPropertyPy {
        SbmlPropertyPy {
            element: property.0.element.clone(),
            name: property.0.name,
            property_namespace: property.0.property_namespace,
            element_namespace: property.0.element_namespace,
            is_required: true,
            converter: T::converter(),
        }
    }

    fn as_sbml_property(&self) -> SbmlProperty<String> {
        SbmlProperty::new(
            &self.element,
            self.name,
            self.property_namespace,
            self.element_namespace,
        )
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl SbmlPropertyPy {
    /// See [XmlProperty::quantified_name].
    #[pyo3(signature = (_write_doc = false))]
    pub fn quantified_name(&self, _write_doc: bool) -> PyResult<String> {
        self.as_sbml_property()
            .quantified_name(false)
            .map_err(runtime_error)
    }

    /// See [XmlProperty::simple_name].
    pub fn simple_name(&self) -> String {
        self.name.to_string()
    }

    /// See [XmlProperty::is_set].
    pub fn is_set(&self) -> bool {
        self.as_sbml_property().is_set()
    }

    /// See [XmlProperty::get_checked].
    ///
    /// Note that [crate::xml::OptionalXmlProperty::get] and [crate::xml::RequiredXmlProperty::get]
    /// also correspond to this function. Since the value type is dynamic, we can simply check
    /// whether the value is required dynamically.
    pub fn get(&self, py: Python) -> PyResult<PyObject> {
        match self.get_raw() {
            Some(value) => self.converter.try_from_string(value, py),
            None => {
                if self.is_required {
                    throw_runtime_error(format!(
                        "Missing value for attribute `{}`.",
                        self.simple_name()
                    ))
                } else {
                    Ok(py.None())
                }
            }
        }
    }

    /// See [XmlProperty::set_raw].
    pub fn set_raw(&self, value: String) -> PyResult<()> {
        self.as_sbml_property()
            .set_raw(value)
            .map_err(runtime_error)
    }

    /// See [XmlProperty::get_raw].
    pub fn get_raw(&self) -> Option<String> {
        self.as_sbml_property().get_raw()
    }

    /// See [XmlProperty::clear]
    pub fn clear(&self) -> PyResult<()> {
        self.as_sbml_property().clear().map_err(runtime_error)
    }

    /// See [crate::xml::OptionalXmlProperty::set] and [crate::xml::RequiredXmlProperty::set].
    pub fn set(&self, py: Python, value: PyObject) -> PyResult<()> {
        if value.is_none(py) {
            if self.is_required {
                throw_type_error(format!(
                    "Required property `{}` cannot accept value `None`.",
                    self.simple_name()
                ))
            } else {
                self.clear()
            }
        } else {
            let value = self.converter.try_into_string(value, py)?;
            self.set_raw(value)
        }
    }
}
