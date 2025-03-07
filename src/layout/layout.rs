use crate::core::sbase::SbmlUtils;
use crate::core::{MetaId, SId};
use crate::layout::bounding_box::BoundingBox;
use crate::layout::curve::Curve;
use crate::layout::dimensions::Dimensions;
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredChild, RequiredProperty, RequiredXmlChild,
    RequiredXmlProperty, XmlDocument, XmlElement, XmlList, XmlNamedSubtype, XmlPropertyType,
    XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Layout(XmlElement);

impl Layout {
    pub fn new(document: XmlDocument, id: SId, dimensions: Dimensions) -> Self {
        let layout = Layout::new_empty(document, "layout");
        layout.id().set(&id);
        layout.dimensions().set(dimensions);
        layout
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn name(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("name")
    }
    pub fn dimensions(&self) -> RequiredChild<Dimensions> {
        self.required_sbml_child("dimensions")
    }
    //can contain only GraphicalObject or GeneralGlyph
    pub fn additional_graph_obj(&self) -> OptionalChild<XmlList<GraphicalObject>> {
        self.optional_sbml_child("additionalGraphObjects")
    }
    pub fn compartment_glyphs(&self) -> OptionalChild<XmlList<CompartmentGlyph>> {
        self.optional_sbml_child("compartmentGlyphs")
    }
    pub fn species_glyph(&self) -> OptionalChild<XmlList<SpeciesGlyph>> {
        self.optional_sbml_child("speciesGlyph")
    }
    pub fn reaction_glyph(&self) -> OptionalChild<XmlList<ReactionGlyph>> {
        self.optional_sbml_child("reactionGlyph")
    }
    pub fn text_glyphs(&self) -> OptionalChild<XmlList<TextGlyph>> {
        self.optional_sbml_child("textGlyphs")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct GraphicalObject(XmlElement);

impl XmlSupertype for GraphicalObject {}

impl GraphicalObject {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox) -> Self {
        let obj = GraphicalObject::new_empty(document, "graphObject");

        obj.id().set(&id);
        obj.bounding_box().set(bounding_box);
        obj
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }
    pub fn annotation(&self) -> OptionalChild<XmlElement> {
        self.optional_sbml_child("annotation")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct CompartmentGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for CompartmentGlyph {
    fn expected_tag_name() -> &'static str {
        "compartmentGlyph"
    }
}

impl CompartmentGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox) -> Self {
        let glyph = CompartmentGlyph::new_empty(document, "compartmentGlyph");
        glyph.id().set(&id);
        glyph.bounding_box().set(bounding_box);
        glyph
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn compartment(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("id")
    }
    pub fn order(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("order")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesGlyph {
    fn expected_tag_name() -> &'static str {
        "speciesGlyph"
    }
}

impl SpeciesGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox) -> Self {
        let glyph = SpeciesGlyph::new_empty(document, "speciesGlyph");
        glyph.id().set(&id);
        glyph.bounding_box().set(bounding_box);
        glyph
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn species(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ReactionGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReactionGlyph {
    fn expected_tag_name() -> &'static str {
        "reactionGlyph"
    }
}

impl ReactionGlyph {
    pub fn new(
        document: XmlDocument,
        id: SId,
        bounding_box: BoundingBox,
        list: XmlList<SpeciesReferenceGlyph>,
    ) -> Self {
        let glyph = ReactionGlyph::new_empty(document, "reactionGlyph");
        glyph.id().set(&id);
        glyph.bounding_box().set(bounding_box);
        glyph.species_reference_glyphs().set(list);
        glyph
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn reaction(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("reaction")
    }
    pub fn species_reference_glyphs(&self) -> RequiredChild<XmlList<SpeciesReferenceGlyph>> {
        self.required_sbml_child("speciesReferenceGlyphs")
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_sbml_child("curve")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesReferenceGlyph {
    fn expected_tag_name() -> &'static str {
        "speciesReferenceGlyph"
    }
}

impl SpeciesReferenceGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox, glyph: SId) -> Self {
        let srg = SpeciesReferenceGlyph::new_empty(document, "speciesRefGlyph");
        srg.glyph().set(&glyph);
        srg.id().set(&id);
        srg.bounding_box().set(bounding_box);
        srg
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn glyph(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("glyph")
    }
    pub fn reference(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("reference")
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_sbml_child("curve")
    }
    pub fn role(&self) -> OptionalProperty<Role> {
        self.optional_sbml_property("role")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Substrate,
    Product,
    Sidesubstrate,
    Sideproduct,
    Modifier,
    Activator,
    Inhibitor,
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

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct GeneralGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for GeneralGlyph {
    fn expected_tag_name() -> &'static str {
        "generalGlyph"
    }
}

impl GeneralGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox) -> Self {
        let gen = GeneralGlyph::new_empty(document, "generalGlyph");
        gen.id().set(&id);
        gen.bounding_box().set(bounding_box);
        gen
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn reference(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("reference")
    }
    pub fn sub_glyphs(&self) -> OptionalChild<XmlList<GraphicalObject>> {
        self.optional_sbml_child("subGlyphs")
    }
    pub fn reference_glyphs(&self) -> OptionalChild<XmlList<ReferenceGlyph>> {
        self.optional_sbml_child("referenceGlyphs")
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_sbml_child("curve")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReferenceGlyph {
    fn expected_tag_name() -> &'static str {
        "referenceGlyph"
    }
}

impl ReferenceGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox, glyph: SId) -> Self {
        let ref_g = ReferenceGlyph::new_empty(document, "referenceGlyph");
        ref_g.id().set(&id);
        ref_g.bounding_box().set(bounding_box);
        ref_g.glyph().set(&glyph);
        ref_g
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn glyph(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("glyph")
    }
    pub fn reference(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("reference")
    }
    pub fn role(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("role")
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_sbml_child("curve")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct TextGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for TextGlyph {
    fn expected_tag_name() -> &'static str {
        "textGlyph"
    }
}

impl TextGlyph {
    pub fn new(document: XmlDocument, id: SId, bounding_box: BoundingBox) -> Self {
        let txt = TextGlyph::new_empty(document, "textGlyph");
        txt.id().set(&id);
        txt.bounding_box().set(bounding_box);
        txt
    }

    pub fn id(&self) -> RequiredProperty<SId> {
        self.required_sbml_property("id")
    }
    pub fn meta_id_ref(&self) -> OptionalProperty<MetaId> {
        self.optional_sbml_property("metaIdRef")
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_sbml_child("boundingBox")
    }

    pub fn graphical_object(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("graphicalObject")
    }
    pub fn text(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("text")
    }
    pub fn origin_of_text(&self) -> OptionalProperty<SId> {
        self.optional_sbml_property("originOfText")
    }
}
