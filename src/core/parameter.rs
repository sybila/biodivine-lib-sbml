use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "parameter", namespace = NS_SBML_CORE)]
pub struct Parameter(XmlElement);

impl Parameter {
    /// Creates a new `<parameter>` element with the given `id`, `constant`.
    pub fn new(document: Document, id: &SId, constant: bool) -> Self {
        let obj = Parameter::new_empty(document);
        obj.id().set(id);
        obj.constant().set(&constant);
        obj
    }

    /// The `id` attribute of this `<parameter>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("id")
    }

    /// The `value` attribute of this `<parameter>` element.
    pub fn value(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("value")
    }

    /// The `units` attribute of this `<parameter>` element.
    pub fn units(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("units")
    }

    /// The `constant` attribute of this `<parameter>` element.
    pub fn constant(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("constant")
    }
}
