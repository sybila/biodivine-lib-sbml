use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalProperty, RequiredProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};
use crate::constants::namespaces::NS_LAYOUT;

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Point(XmlElement);

impl Point {
    pub fn new(document: XmlDocument, x: f64, y: f64) -> Self {
        let obj = Point::new_empty(document, "point");
        obj.x().set(&x);
        obj.y().set(&y);
        obj
    }

    pub fn id(&self) -> OptionalProperty<SId> {
        self.optional_package_property("id", NS_LAYOUT, false)
    }

    pub fn x(&self) -> RequiredProperty<f64> {
        self.required_package_property("x", NS_LAYOUT, false)
    }

    pub fn y(&self) -> RequiredProperty<f64> {
        self.required_package_property("y", NS_LAYOUT, false)
    }

    pub fn z(&self) -> OptionalProperty<f64> {
        self.optional_package_property("z", NS_LAYOUT, false)
    }
}
