use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "dimensions", namespace = NS_LAYOUT)]
pub struct Dimensions(XmlElement);

impl Dimensions {
    pub fn new(document: Document, width: f64, height: f64) -> Self {
        let dim = Dimensions::new_empty(document);
        dim.width().set(&width);
        dim.height().set(&height);
        dim
    }

    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn width(&self) -> RequiredSbmlProperty<f64> {
        self.required_package_property("width", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn height(&self) -> RequiredSbmlProperty<f64> {
        self.required_package_property("height", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn depth(&self) -> OptionalSbmlProperty<f64> {
        self.optional_package_property("depth", NS_LAYOUT, NS_LAYOUT)
    }
}
