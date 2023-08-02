use crate::xml::{XmlElement, XmlWrapper};
use macros::{XmlChild, XmlProperty};

// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

#[derive(XmlProperty)]
#[property_name("id")]
#[property_type(Option<String>)]
pub struct Id<'a>(&'a XmlElement);

#[derive(XmlProperty)]
#[property_name("name")]
#[property_type(Option<String>)]
pub struct Name<'a>(&'a XmlElement);

#[derive(XmlProperty)]
#[property_name("metaid")]
#[property_type(Option<String>)]
pub struct MetaId<'a>(&'a XmlElement);

#[derive(XmlProperty)]
#[property_name("sboTerm")]
#[property_type(Option<String>)]
pub struct SboTerm<'a>(&'a XmlElement);

#[derive(XmlChild)]
#[child_name("notes")]
#[child_type(XmlElement)]
pub struct Notes<'a>(&'a XmlElement);

#[derive(XmlChild)]
#[child_name("annotation")]
#[child_type(XmlElement)]
pub struct Annotation<'a>(&'a XmlElement);

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> Id {
        Id::for_element(self.as_xml())
    }

    fn name(&self) -> Name {
        Name::for_element(self.as_xml())
    }

    fn meta_id(&self) -> MetaId {
        MetaId::for_element(self.as_xml())
    }

    fn sbo_term(&self) -> SboTerm {
        SboTerm::for_element(self.as_xml())
    }

    fn notes(&self) -> Notes {
        Notes::for_element(self.as_xml())
    }

    fn annotation(&self) -> Annotation {
        Annotation::for_element(self.as_xml())
    }
}
