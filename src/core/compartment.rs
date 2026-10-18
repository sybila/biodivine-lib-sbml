use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDefault,
    XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// Individual compartment definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "compartment", namespace = NS_SBML_CORE)]
pub struct Compartment(XmlElement);

impl XmlDefault for Compartment {
    fn default(document: Document) -> Self {
        Compartment::new(document, true)
    }
}

impl Compartment {
    /// Creates a new `<compartment>` element with the given `is_constant`.
    pub fn new(document: Document, is_constant: bool) -> Self {
        let cmp = Compartment::new_empty(document);
        cmp.constant().set(&is_constant);
        cmp
    }

    /// The `id` attribute of this `<compartment>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("id")
    }

    /// The `spatialDimensions` attribute of this `<compartment>` element.
    pub fn spatial_dimensions(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("spatialDimensions")
    }

    /// The `size` attribute of this `<compartment>` element.
    pub fn size(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("units")
    }

    /// The `constant` attribute of this `<compartment>` element.
    pub fn constant(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("constant")
    }
}
