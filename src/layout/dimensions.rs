use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// The `<dimensions>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "dimensions", namespace = NS_LAYOUT)]
pub struct Dimensions(XmlElement);

impl Dimensions {
    /// Creates a new `<dimensions>` element with the given `width`, `height`.
    pub fn new(document: Document, width: f64, height: f64) -> Self {
        let dim = Dimensions::new_empty(document);
        dim.width().set(&width);
        dim.height().set(&height);
        dim
    }

    /// The `id` attribute of this `<dimensions>` element.
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `width` attribute of this `<dimensions>` element.
    pub fn width(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_package_property("width", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `height` attribute of this `<dimensions>` element.
    pub fn height(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_package_property("height", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `depth` attribute of this `<dimensions>` element.
    pub fn depth(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_package_property("depth", NS_LAYOUT, NS_LAYOUT)
    }
}
