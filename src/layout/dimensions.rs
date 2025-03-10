use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalProperty, RequiredProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Dimensions(XmlElement);

impl Dimensions {
    pub fn new(document: XmlDocument, width: f64, height: f64) -> Self {
        let dim = Dimensions::new_empty(document, "dimensions");
        dim.width().set(&width);
        dim.height().set(&height);
        dim
    }

    pub fn id(&self) -> OptionalProperty<SId> {
        self.optional_package_property("id", NS_LAYOUT, false)
    }
    pub fn width(&self) -> RequiredProperty<f64> {
        self.required_package_property("width", NS_LAYOUT, false)
    }
    pub fn height(&self) -> RequiredProperty<f64> {
        self.required_package_property("height", NS_LAYOUT, false)
    }
    pub fn depth(&self) -> OptionalProperty<f64> {
        self.optional_package_property("depth", NS_LAYOUT, false)
    }
}
