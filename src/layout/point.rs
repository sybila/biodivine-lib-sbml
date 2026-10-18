use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// The point of a Layout package diagram: the `<position>` of a bounding box, and the `<start>`,
/// `<end>`, `<basePoint1>` and `<basePoint2>` of a curve segment.
///
/// The five names share this wrapper because they are the same shape (a point in the diagram's
/// coordinate system); which one an element is decides where it may appear, which the validation
/// tables check.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["position", "start", "end", "basePoint1", "basePoint2"],
    namespace = NS_LAYOUT
)]
pub struct Point(XmlElement);

impl Point {
    /// Creates a new `<position>` element (the point of a bounding box) with the given `x` and `y`.
    pub fn new(document: Document, x: f64, y: f64) -> Self {
        let obj = Point::new_empty(document);
        obj.x().set(&x);
        obj.y().set(&y);
        obj
    }

    /// The `id` attribute of this point element.
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }

    /// The `x` attribute of this point element.
    pub fn x(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_package_property("x", NS_LAYOUT, NS_LAYOUT)
    }

    /// The `y` attribute of this point element.
    pub fn y(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_package_property("y", NS_LAYOUT, NS_LAYOUT)
    }

    /// The `z` attribute of this point element.
    pub fn z(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_package_property("z", NS_LAYOUT, NS_LAYOUT)
    }
}
