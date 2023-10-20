use crate::constants::namespaces::URL_SBML_CORE;
use crate::xml::{OptionalChild, OptionalProperty, XmlElement, XmlWrapper};

// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xmlElement(), "id")
    }

    fn name(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xmlElement(), "name")
    }

    fn meta_id(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xmlElement(), "metaid")
    }

    fn sbo_term(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xmlElement(), "sboTerm")
    }

    fn notes(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(self.xmlElement(), "notes", URL_SBML_CORE)
    }

    fn annotation(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(self.xmlElement(), "annotation", URL_SBML_CORE)
    }
}
