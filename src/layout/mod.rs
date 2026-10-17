mod bounding_box;
mod curve;
mod dimensions;
mod point;
mod validation;

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

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "layout", namespace = NS_LAYOUT)]
pub struct Layout(XmlElement);

impl Layout {
    pub fn new(document: Document, id: SId, dimensions: Dimensions) -> Self {
        let layout = Layout::new_empty(document);
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
    pub fn dimensions(&self) -> RequiredSbmlChild<Dimensions> {
        self.required_package_child("dimensions", NS_LAYOUT, false)
    }
    pub fn additional_graph_obj(&self) -> OptionalSbmlChild<XmlList<GraphicalObject>> {
        self.optional_package_child("listOfAdditionalGraphicalObjects", NS_LAYOUT, false)
    }
    pub fn compartment_glyphs(&self) -> OptionalSbmlChild<XmlList<CompartmentGlyph>> {
        self.optional_package_child("listOfCompartmentGlyphs", NS_LAYOUT, false)
    }
    pub fn species_glyphs(&self) -> OptionalSbmlChild<XmlList<SpeciesGlyph>> {
        self.optional_package_child("listOfSpeciesGlyphs", NS_LAYOUT, false)
    }
    pub fn reaction_glyphs(&self) -> OptionalSbmlChild<XmlList<ReactionGlyph>> {
        self.optional_package_child("listOfReactionGlyphs", NS_LAYOUT, false)
    }
    pub fn text_glyphs(&self) -> OptionalSbmlChild<XmlList<TextGlyph>> {
        self.optional_package_child("listOfTextGlyphs", NS_LAYOUT, false)
    }
}

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
    pub fn new(document: Document, id: SId, bounding_box: BoundingBox) -> Self {
        let obj = GraphicalObject::new_empty(document);

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
    pub fn bounding_box(&self) -> RequiredSbmlChild<BoundingBox> {
        self.required_package_child("boundingBox", NS_LAYOUT, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "compartmentGlyph", namespace = NS_LAYOUT)]
pub struct CompartmentGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for CompartmentGlyph {}

impl CompartmentGlyph {
    pub fn new(document: Document, id: SId) -> Self {
        let glyph = CompartmentGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn compartment(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("compartment", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn order(&self) -> OptionalSbmlProperty<f64> {
        self.optional_package_property("order", NS_LAYOUT, NS_LAYOUT)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "speciesGlyph", namespace = NS_LAYOUT)]
pub struct SpeciesGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesGlyph {}

impl SpeciesGlyph {
    pub fn new(document: Document, id: SId) -> Self {
        let glyph = SpeciesGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn species(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("species", NS_LAYOUT, NS_LAYOUT)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "reactionGlyph", namespace = NS_LAYOUT)]
pub struct ReactionGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReactionGlyph {}

impl ReactionGlyph {
    pub fn new(document: Document, id: SId, list: XmlList<SpeciesReferenceGlyph>) -> Self {
        let glyph = ReactionGlyph::new_empty(document);
        glyph.id().set(&id);
        glyph.species_reference_glyphs().set(list);
        glyph
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn reaction(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("reaction", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn species_reference_glyphs(&self) -> RequiredSbmlChild<XmlList<SpeciesReferenceGlyph>> {
        self.required_package_child("listOfSpeciesReferenceGlyphs", NS_LAYOUT, false)
    }
    pub fn curve(&self) -> OptionalSbmlChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "speciesReferenceGlyph", namespace = NS_LAYOUT)]
pub struct SpeciesReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for SpeciesReferenceGlyph {}

impl SpeciesReferenceGlyph {
    pub fn new(document: Document, id: SId, glyph: SId) -> Self {
        let srg = SpeciesReferenceGlyph::new_empty(document);
        srg.species_glyph().set(&glyph);
        srg.id().set(&id);
        srg
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn species_glyph(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("speciesGlyph", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn species_reference(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("speciesReference", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn curve(&self) -> OptionalSbmlChild<Curve> {
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
#[sbml_object(name = "generalGlyph", namespace = NS_LAYOUT)]
pub struct GeneralGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for GeneralGlyph {}

impl GeneralGlyph {
    pub fn new(document: Document, id: SId) -> Self {
        let gen = GeneralGlyph::new_empty(document);
        gen.id().set(&id);
        gen
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
    }

    pub fn reference(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("reference", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn sub_glyphs(&self) -> OptionalSbmlChild<XmlList<GraphicalObject>> {
        self.optional_package_child("listOfSubGlyphs", NS_LAYOUT, false)
    }
    pub fn reference_glyphs(&self) -> OptionalSbmlChild<XmlList<ReferenceGlyph>> {
        self.optional_package_child("listOfReferenceGlyphs", NS_LAYOUT, false)
    }
    pub fn curve(&self) -> OptionalSbmlChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "referenceGlyph", namespace = NS_LAYOUT)]
pub struct ReferenceGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for ReferenceGlyph {}

impl ReferenceGlyph {
    pub fn new(document: Document, id: SId, glyph: SId) -> Self {
        let ref_g = ReferenceGlyph::new_empty(document);
        ref_g.id().set(&id);
        ref_g.glyph().set(&glyph);
        ref_g
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
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
    pub fn curve(&self) -> OptionalSbmlChild<Curve> {
        self.optional_package_child("curve", NS_LAYOUT, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "textGlyph", namespace = NS_LAYOUT)]
pub struct TextGlyph(XmlElement);

impl XmlNamedSubtype<GraphicalObject> for TextGlyph {}

impl TextGlyph {
    pub fn new(document: Document, id: SId) -> Self {
        let txt = TextGlyph::new_empty(document);
        txt.id().set(&id);
        txt
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn meta_id_ref(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_package_property("metaidRef", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn bounding_box(&self) -> OptionalSbmlChild<BoundingBox> {
        self.optional_package_child("boundingBox", NS_LAYOUT, false)
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
