use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlDocument,
    XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// Individual compartment definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Compartment(XmlElement);

impl XmlDefault for Compartment {
    fn default(document: XmlDocument) -> Self {
        Compartment::new(document, true)
    }
}

impl Compartment {
    pub fn new(document: XmlDocument, is_constant: bool) -> Self {
        let cmp = Compartment::new_empty(document, "compartment");
        cmp.constant().set(&is_constant);
        cmp
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("id")
    }

    pub fn spatial_dimensions(&self) -> OptionalSbmlProperty<f64> {
        self.optional_sbml_property("spatialDimensions")
    }

    pub fn size(&self) -> OptionalSbmlProperty<f64> {
        self.optional_sbml_property("size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("constant")
    }
}
