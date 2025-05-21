use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::layout::dimensions::Dimensions;
use crate::layout::point::Point;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlChild, RequiredXmlChild, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct BoundingBox(XmlElement);

impl BoundingBox {
    pub fn new(document: XmlDocument, position: Point, dimensions: Dimensions) -> Self {
        let b_box = BoundingBox::new_empty(document, "bounding_box");
        b_box.position().set(position);
        b_box.dimensions().set(dimensions);
        b_box
    }

    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn position(&self) -> RequiredSbmlChild<Point> {
        self.required_package_child("position", NS_LAYOUT, false)
    }
    pub fn dimensions(&self) -> RequiredSbmlChild<Dimensions> {
        self.required_package_child("dimensions", NS_LAYOUT, false)
    }
}
