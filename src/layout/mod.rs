mod bounding_box;
mod curve;
mod dimensions;
mod point;
pub(crate) mod validation;

use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::{MetaId, SId};
use crate::xml::{
    Document, OptionalSbmlChild, OptionalSbmlProperty, RequiredSbmlChild, RequiredSbmlProperty,
    RequiredXmlChild, RequiredXmlProperty, XmlElement, XmlList, XmlNamedSubtype, XmlPropertyType,
    XmlSupertype,
};
pub use bounding_box::BoundingBox;
pub use curve::{CubicBezier, Curve, LineSegment, XsiType};
pub use dimensions::Dimensions;
pub use point::Point;
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

/// The `<layout>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "layout", namespace = NS_LAYOUT)]
pub struct Layout(XmlElement);

impl Layout {
    /// Creates a new `<layout>` element with the given `id`, `dimensions`.
    pub fn new(document: Document, id: SId, dimensions: Dimensions) -> Self {
        let layout = Layout::new_empty(document);
        layout.id().set(&id);
        layout.dimensions().set(dimensions);
        layout
    }

    /// The `id` attribute of this `<layout>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `name` attribute of this `<layout>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `dimensions` child of this `<layout>` element.
    pub fn dimensions(&self) -> RequiredSbmlChild<'_, Dimensions> {
        self.required_package_child("dimensions", NS_LAYOUT, false)
    }
    /// The `listOfAdditionalGraphicalObjects` child of this `<layout>` element, as a list of [`GraphicalObject`].
    pub fn additional_graph_obj(&self) -> OptionalSbmlChild<'_, XmlList<GraphicalObject>> {
        self.optional_package_child("listOfAdditionalGraphicalObjects", NS_LAYOUT, false)
    }
    /// The `listOfCompartmentGlyphs` child of this `<layout>` element, as a list of [`CompartmentGlyph`].
    pub fn compartment_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<CompartmentGlyph>> {
        self.optional_package_child("listOfCompartmentGlyphs", NS_LAYOUT, false)
    }
    /// The `listOfSpeciesGlyphs` child of this `<layout>` element, as a list of [`SpeciesGlyph`].
    pub fn species_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<SpeciesGlyph>> {
        self.optional_package_child("listOfSpeciesGlyphs", NS_LAYOUT, false)
    }
    /// The `listOfReactionGlyphs` child of this `<layout>` element, as a list of [`ReactionGlyph`].
    pub fn reaction_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<ReactionGlyph>> {
        self.optional_package_child("listOfReactionGlyphs", NS_LAYOUT, false)
    }
    /// The `listOfTextGlyphs` child of this `<layout>` element, as a list of [`TextGlyph`].
    pub fn text_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<TextGlyph>> {
        self.optional_package_child("listOfTextGlyphs", NS_LAYOUT, false)
    }
}

/// The `<layout:graphicalObject>` element: the supertype of every glyph of the Layout package, a
/// shape with an identifier, a bounding box and an optional reference to a model element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = [
        "graphicalObject",
        "compartmentGlyph",
        "generalGlyph",
        "reactionGlyph",
        "referenceGlyph",
        "speciesGlyph",
        "speciesReferenceGlyph",
        "textGlyph"
    ],
    namespace = NS_LAYOUT
)]
pub struct GraphicalObject(XmlElement);

impl XmlSupertype for GraphicalObject {}

impl GraphicalObject {
    /// Creates a new `<layout:graphicalObject>` element with the given identifier and bounding box.
    pub fn new(document: Document, id: SId, bounding_box: BoundingBox) -> Self {
        let obj = GraphicalObject::new_empty(document);

        obj.id().set(&id);
        obj.bounding_box().set(bounding_box);
        obj
    }

    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute: the `metaid` of the model element this glyph draws.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child: the rectangle the glyph occupies.
    pub fn bounding_box(&self) -> RequiredSbmlChild<'_, BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }
}

/// The `<compartmentGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "compartmentGlyph", namespace = NS_LAYOUT)]
pub struct CompartmentGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for CompartmentGlyph {}

impl CompartmentGlyph {
    /// Creates a new `<compartmentGlyph>` element with the given `id`.
    pub fn new(document: Document, id: SId) -> Self {
        let glyph = CompartmentGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph
    }

    /// The `id` attribute of this `<compartmentGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<compartmentGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<compartmentGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `compartment` attribute of this `<compartmentGlyph>` element.
    pub fn compartment(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("compartment", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `order` attribute of this `<compartmentGlyph>` element.
    pub fn order(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_package_property("order", NS_LAYOUT, NS_LAYOUT)
    }
}

/// The `<speciesGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "speciesGlyph", namespace = NS_LAYOUT)]
pub struct SpeciesGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesGlyph {}

impl SpeciesGlyph {
    /// Creates a new `<speciesGlyph>` element with the given `id`.
    pub fn new(document: Document, id: SId) -> Self {
        let glyph = SpeciesGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph
    }

    /// The `id` attribute of this `<speciesGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<speciesGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<speciesGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `species` attribute of this `<speciesGlyph>` element.
    pub fn species(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("species", NS_LAYOUT, NS_LAYOUT)
    }
}

/// The `<reactionGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "reactionGlyph", namespace = NS_LAYOUT)]
pub struct ReactionGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReactionGlyph {}

impl ReactionGlyph {
    /// Creates a new `<reactionGlyph>` element with the given `id`, `list`.
    pub fn new(document: Document, id: SId, list: XmlList<SpeciesReferenceGlyph>) -> Self {
        let glyph = ReactionGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph.species_reference_glyphs().set(list);
        glyph
    }

    /// The `id` attribute of this `<reactionGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<reactionGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<reactionGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `reaction` attribute of this `<reactionGlyph>` element.
    pub fn reaction(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("reaction", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `listOfSpeciesReferenceGlyphs` child of this `<reactionGlyph>` element.
    pub fn species_reference_glyphs(
        &self,
    ) -> RequiredSbmlChild<'_, XmlList<SpeciesReferenceGlyph>> {
        self.required_package_child("listOfSpeciesReferenceGlyphs", NS_LAYOUT, false)
    }
    /// The `curve` child of this `<reactionGlyph>` element.
    pub fn curve(&self) -> OptionalSbmlChild<'_, Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

/// The `<speciesReferenceGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "speciesReferenceGlyph", namespace = NS_LAYOUT)]
pub struct SpeciesReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesReferenceGlyph {}

impl SpeciesReferenceGlyph {
    /// Creates a new `<speciesReferenceGlyph>` element with the given `id`, `glyph`.
    pub fn new(document: Document, id: SId, glyph: SId) -> Self {
        let srg = SpeciesReferenceGlyph::new_empty(document);
        srg.species_glyph().set(&glyph);
        srg.id().set(&id);
        srg
    }

    /// The `id` attribute of this `<speciesReferenceGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<speciesReferenceGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<speciesReferenceGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `speciesGlyph` attribute of this `<speciesReferenceGlyph>` element.
    pub fn species_glyph(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("speciesGlyph", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `speciesReference` attribute of this `<speciesReferenceGlyph>` element.
    pub fn species_reference(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("speciesReference", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `curve` child of this `<speciesReferenceGlyph>` element.
    pub fn curve(&self) -> OptionalSbmlChild<'_, Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
    /// The `role` attribute of this `<speciesReferenceGlyph>` element.
    pub fn role(&self) -> OptionalSbmlProperty<'_, Role> {
        self.optional_package_property("role", NS_LAYOUT, NS_LAYOUT)
    }
}

/// The `layout:role` of a species reference glyph (Layout Package, Section 3.6): what the species
/// does in the reaction the glyph belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    /// The species is a substrate of the reaction.
    Substrate,
    /// The species is a product of the reaction.
    Product,
    /// The species is a substrate that is not a main reactant of the reaction.
    Sidesubstrate,
    /// The species is a product that is not a main product of the reaction.
    Sideproduct,
    /// The species modifies the reaction without being consumed or produced.
    Modifier,
    /// The species activates the reaction.
    Activator,
    /// The species inhibits the reaction.
    Inhibitor,
    /// The role of the species is not specified.
    Undefined,
}

impl TryFrom<String> for Role {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "substrate" => Ok(Role::Substrate),
            "product" => Ok(Role::Product),
            "sidesubstrate" => Ok(Role::Sidesubstrate),
            "sideproduct" => Ok(Role::Sidesubstrate),
            "modifier" => Ok(Role::Modifier),
            "activator" => Ok(Role::Activator),
            "inhibitor" => Ok(Role::Inhibitor),
            "undefined" => Ok(Role::Undefined),
            _ => Err(format!("Role '{value}' is not valid role type")),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Role::Substrate => "Substrate",
            Role::Product => "Product",
            Role::Sidesubstrate => "Sidesubstrate",
            Role::Sideproduct => "Sideproduct",
            Role::Modifier => "Modifier",
            Role::Activator => "Activator",
            Role::Inhibitor => "Inhibitor",
            Role::Undefined => "Undefined",
        };

        write!(f, "{}", str)
    }
}

impl XmlPropertyType for Role {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match Role::try_from(value.to_string()) {
                Ok(role) => Ok(Some(role)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}

/// The `<generalGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "generalGlyph", namespace = NS_LAYOUT)]
pub struct GeneralGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for GeneralGlyph {}

impl GeneralGlyph {
    /// Creates a new `<generalGlyph>` element with the given `id`.
    pub fn new(document: Document, id: SId) -> Self {
        let gen = GeneralGlyph::new_empty(document);
        gen.id().set(&id);
        gen
    }

    /// The `id` attribute of this `<generalGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<generalGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<generalGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `reference` attribute of this `<generalGlyph>` element.
    pub fn reference(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("reference", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `listOfSubGlyphs` child of this `<generalGlyph>` element, as a list of [`GraphicalObject`].
    pub fn sub_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<GraphicalObject>> {
        self.optional_package_child("listOfSubGlyphs", NS_LAYOUT, false)
    }
    /// The `listOfReferenceGlyphs` child of this `<generalGlyph>` element, as a list of [`ReferenceGlyph`].
    pub fn reference_glyphs(&self) -> OptionalSbmlChild<'_, XmlList<ReferenceGlyph>> {
        self.optional_package_child("listOfReferenceGlyphs", NS_LAYOUT, false)
    }
    /// The `curve` child of this `<generalGlyph>` element.
    pub fn curve(&self) -> OptionalSbmlChild<'_, Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

/// The `<referenceGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "referenceGlyph", namespace = NS_LAYOUT)]
pub struct ReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReferenceGlyph {}

impl ReferenceGlyph {
    /// Creates a new `<referenceGlyph>` element with the given `id`, `glyph`.
    pub fn new(document: Document, id: SId, glyph: SId) -> Self {
        let ref_g = ReferenceGlyph::new_empty(document);
        ref_g.id().set(&id);
        ref_g.glyph().set(&glyph);
        ref_g
    }

    /// The `id` attribute of this `<referenceGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<referenceGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<referenceGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `glyph` attribute of this `<referenceGlyph>` element.
    pub fn glyph(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("glyph", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `reference` attribute of this `<referenceGlyph>` element.
    pub fn reference(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("reference", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `role` attribute of this `<referenceGlyph>` element.
    pub fn role(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("role", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `curve` child of this `<referenceGlyph>` element.
    pub fn curve(&self) -> OptionalSbmlChild<'_, Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

/// The `<textGlyph>` element of the Layout package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "textGlyph", namespace = NS_LAYOUT)]
pub struct TextGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for TextGlyph {}

impl TextGlyph {
    /// Creates a new `<textGlyph>` element with the given `id`.
    pub fn new(document: Document, id: SId) -> Self {
        let txt = TextGlyph::new_empty(document);
        txt.id().set(&id);
        txt
    }

    /// The `id` attribute of this `<textGlyph>` element.
    /// The `id` attribute every glyph of the Layout package carries.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `metaidRef` attribute of this `<textGlyph>` element.
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<'_, MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `boundingBox` child of this `<textGlyph>` element.
    pub fn bounding_box(&self) -> OptionalSbmlChild<'_, BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    /// The `graphicalObject` attribute of this `<textGlyph>` element.
    pub fn graphical_object(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("graphicalObject", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `text` attribute of this `<textGlyph>` element.
    pub fn text(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("text", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `originOfText` attribute of this `<textGlyph>` element.
    pub fn origin_of_text(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("originOfText", NS_LAYOUT, NS_LAYOUT)
    }
}

#[cfg(test)]
mod test {
    use crate::layout::curve::XsiType;
    use crate::layout::Role;
    use crate::tests::sid;
    use crate::xml::{
        OptionalXmlChild, OptionalXmlProperty, RequiredXmlChild, RequiredXmlProperty,
    };
    use crate::Sbml;

    #[test]
    fn layout_example() {
        let doc = Sbml::read_path("test-inputs/test-layout/layout_example.xml").unwrap();
        let issues = doc.validate().unwrap();
        assert!(issues.is_empty());

        let model = doc.model().get().unwrap();
        let layouts = model.layouts().get().unwrap();
        let one_layout = layouts.get(0);

        assert_eq!(one_layout.id().get(), sid("layout1"));
        assert_eq!(one_layout.dimensions().get().height().get(), f64::from(600));
        assert_eq!(
            one_layout
                .additional_graph_obj()
                .get()
                .unwrap()
                .get(0)
                .id()
                .get(),
            sid("gg0")
        );
    }

    #[test]
    fn compartment_glyph() {
        let doc = Sbml::read_path("test-inputs/test-layout/layout_example.xml").unwrap();
        assert!(doc.validate().unwrap().is_empty());

        let model = doc.model().get().unwrap();
        let layouts = model.layouts().get().unwrap();
        let comp_glyph = layouts.get(0).compartment_glyphs().get().unwrap().get(0);

        assert_eq!(comp_glyph.id().get(), sid("cg0"));
        assert_eq!(comp_glyph.compartment().get().unwrap(), sid("default"));
        assert_eq!(
            comp_glyph
                .bounding_box()
                .get()
                .unwrap()
                .position()
                .get()
                .x()
                .get(),
            f64::from(10)
        );
    }

    #[test]
    fn reaction_glyphs() {
        let doc = Sbml::read_path("test-inputs/test-layout/layout_example.xml").unwrap();
        assert!(doc.validate().unwrap().is_empty());

        let model = doc.model().get().unwrap();
        let layouts = model.layouts().get().unwrap();
        let reactions = layouts.get(0).reaction_glyphs().get().unwrap();

        assert_eq!(reactions.get(0).id().get(), sid("rg0"));
        let sp_references = reactions.get(0).species_reference_glyphs().get();

        assert_eq!(reactions.get(2).id().get(), sid("rg2"));
        assert_eq!(reactions.get(1).reaction().get().unwrap(), sid("r1"));
        assert_eq!(sp_references.get(0).role().get().unwrap(), Role::Substrate);

        let curve1 = reactions.get(0).curve().get().unwrap();
        let curve2 = reactions.get(1).curve().get().unwrap();

        assert_eq!(
            curve1.curve_segments().get().get(0).start().get().x().get(),
            f64::from(170)
        );
        assert_eq!(
            curve2.curve_segments().get().get(0).xsi_type().get(),
            XsiType::CubicBezier
        );
    }
}
