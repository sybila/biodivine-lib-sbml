use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["position", "start", "end", "basePoint1", "basePoint2"],
    namespace = NS_LAYOUT
)]
pub struct Point(XmlElement);

impl Point {
    pub fn new(document: Document, x: f64, y: f64) -> Self {
        let obj = Point::new_empty(document, "point");
        obj.x().set(&x);
        obj.y().set(&y);
        obj
    }

    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }

    pub fn x(&self) -> RequiredSbmlProperty<f64> {
        self.required_package_property("x", NS_LAYOUT, NS_LAYOUT)
    }

    pub fn y(&self) -> RequiredSbmlProperty<f64> {
        self.required_package_property("y", NS_LAYOUT, NS_LAYOUT)
    }

    pub fn z(&self) -> OptionalSbmlProperty<f64> {
        self.optional_package_property("z", NS_LAYOUT, NS_LAYOUT)
    }
}
