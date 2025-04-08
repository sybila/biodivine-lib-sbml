mod bounding_box;
mod curve;
mod dimensions;
mod point;
mod validation;

use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::core::{MetaId, SId};
use crate::layout::bounding_box::BoundingBox;
use crate::layout::curve::Curve;
use crate::layout::dimensions::Dimensions;
use crate::xml::{
    OptionalChild, OptionalSbmlProperty, RequiredChild, RequiredSbmlProperty, RequiredXmlChild,
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn dimensions(&self) -> RequiredChild<Dimensions> {
        self.required_package_child("dimensions", NS_LAYOUT, false)
    }
    //can contain only GraphicalObject or GeneralGlyph
    pub fn additional_graph_obj(&self) -> OptionalChild<XmlList<GraphicalObject>> {
        self.optional_package_child("listOfAdditionalGraphicalObjects", NS_LAYOUT, false)
    }
    pub fn compartment_glyphs(&self) -> OptionalChild<XmlList<CompartmentGlyph>> {
        self.optional_package_child("listOfCompartmentGlyphs", NS_LAYOUT, false)
    }
    pub fn species_glyphs(&self) -> OptionalChild<XmlList<SpeciesGlyph>> {
        self.optional_package_child("listOfSpeciesGlyphs", NS_LAYOUT, false)
    }
    pub fn reaction_glyphs(&self) -> OptionalChild<XmlList<ReactionGlyph>> {
        self.optional_package_child("listOfReactionGlyphs", NS_LAYOUT, false)
    }
    pub fn text_glyphs(&self) -> OptionalChild<XmlList<TextGlyph>> {
        self.optional_package_child("listOfTextGlyphs", NS_LAYOUT, false)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn compartment(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("compartment", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn order(&self) -> OptionalSbmlProperty<f64> {
        self.optional_package_property("order", NS_LAYOUT, NS_LAYOUT)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn species(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("species", NS_LAYOUT, NS_LAYOUT)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn reaction(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("reaction", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn species_reference_glyphs(&self) -> RequiredChild<XmlList<SpeciesReferenceGlyph>> {
        self.required_package_child("speciesReferenceGlyphs", NS_LAYOUT, false)
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
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
        srg.species_glyph().set(&glyph);
        srg.id().set(&id);
        srg.bounding_box().set(bounding_box);
        srg
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn species_glyph(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("speciesGlyph", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn species_reference(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("speciesReference", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
    pub fn role(&self) -> OptionalSbmlProperty<Role> {
        self.optional_package_property("role", NS_LAYOUT, NS_LAYOUT)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn reference(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("reference", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn sub_glyphs(&self) -> OptionalChild<XmlList<GraphicalObject>> {
        self.optional_package_child("listOfSubGlyphs", NS_LAYOUT, false)
    }
    pub fn reference_glyphs(&self) -> OptionalChild<XmlList<ReferenceGlyph>> {
        self.optional_package_child("listOfReferenceGlyphs", NS_LAYOUT, false)
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn glyph(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("glyph", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn reference(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("reference", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn role(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("role", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn curve(&self) -> OptionalChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
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

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> RequiredChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn graphical_object(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("graphicalObject", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn text(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("text", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn origin_of_text(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("originOfText", NS_LAYOUT, NS_LAYOUT)
    }
}
