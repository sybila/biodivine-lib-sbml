use crate::xml::{OptionalChild, OptionalProperty, XmlElement, XmlWrapper};
use crate::URL_SBML_CORE;

// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.as_xml(), "id")
    }

    fn name(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.as_xml(), "name")
    }

    fn meta_id(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.as_xml(), "metaid")
    }

    fn sbo_term(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.as_xml(), "sboTerm")
    }

    fn notes(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(self.as_xml(), "notes", URL_SBML_CORE)
    }

    fn annotation(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(self.as_xml(), "annotation", URL_SBML_CORE)
    }
}
