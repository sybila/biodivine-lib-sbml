use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::layout::dimensions::Dimensions;
use crate::layout::point::Point;
use crate::xml::{Document, OptionalSbmlProperty, RequiredSbmlChild, RequiredXmlChild, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "boundingBox", namespace = NS_LAYOUT)]
pub struct BoundingBox(XmlElement);

impl BoundingBox {
    pub fn new(document: Document, position: Point, dimensions: Dimensions) -> Self {
        let b_box = BoundingBox::new_empty(document);
        b_box.position().set(position);
        b_box.dimensions().set(dimensions);
        b_box
    }

    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn position(&self) -> RequiredSbmlChild<'_, Point> {
        self.required_package_child("position", NS_LAYOUT, false)
    }
    pub fn dimensions(&self) -> RequiredSbmlChild<'_, Dimensions> {
        self.required_package_child("dimensions", NS_LAYOUT, false)
    }
}
