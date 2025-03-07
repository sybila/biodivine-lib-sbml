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
        self.optional_sbml_property("id")
    }
    pub fn width(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("width")
    }
    pub fn height(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("height")
    }
    pub fn depth(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("depth")
    }
}
