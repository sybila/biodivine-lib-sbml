use crate::xml::impl_xml_child::Child;
use crate::xml::impl_xml_property::Property;
use crate::xml::{XmlElement, XmlWrapper};

// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "id")
    }

    fn name(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "name")
    }

    fn meta_id(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "metaid")
    }

    fn sbo_term(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "sboTerm")
    }

    fn notes(&self) -> Child<XmlElement> {
        Child::new(self.as_xml(), "notes")
    }

    fn annotation(&self) -> Child<XmlElement> {
        Child::new(self.as_xml(), "annotation")
    }
}
