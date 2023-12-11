use crate::core::sbase::SbmlUtils;
use crate::xml::{OptionalProperty, RequiredProperty, XmlDefault, XmlDocument, XmlElement};
use macros::{SBase, XmlWrapper};

/// Individual compartment definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Compartment(XmlElement);

impl XmlDefault for Compartment {
    fn default(document: XmlDocument) -> Self {
        Compartment::new_empty(document, "compartment")
    }
}

impl Compartment {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn spatial_dimensions(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("spatialDimensions")
    }

    pub fn size(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}
