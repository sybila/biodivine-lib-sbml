use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalProperty, RequiredProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

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
        self.optional_sbml_property("id")
    }

    pub fn x(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("x")
    }

    pub fn y(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("y")
    }

    pub fn z(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("z")
    }
}
