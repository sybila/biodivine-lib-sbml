use crate::constants::namespaces::{NS_SBML_CORE, URL_HTML, URL_MATHML, URL_SBML_CORE};
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredProperty, XmlDocument, XmlElement, XmlPropertyType,
    XmlWrapper,
};

// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("id")
    }

    fn name(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("name")
    }

    fn meta_id(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("metaid")
    }

    fn sbo_term(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("sboTerm")
    }

    fn notes(&self) -> OptionalChild<XmlElement> {
        self.optional_sbml_child("notes")
    }

    fn annotation(&self) -> OptionalChild<XmlElement> {
        self.optional_sbml_child("annotation")
    }
}

/// TODO:
///     In the end, this trait probably should not be accessible from the outside, but we can
///     discuss this later.
pub(crate) trait SbmlUtils: SBase {
    /// Create a new instance of `Self` which is just an empty tag with the given `tag_name`
    /// and using SBML namespace.
    #[inline(always)]
    fn new_empty(document: XmlDocument, tag_name: &str) -> Self {
        unsafe {
            let element = XmlElement::new_quantified(document, tag_name, NS_SBML_CORE);
            Self::unchecked_cast(element)
        }
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the SBML namespace.
    #[inline(always)]
    fn optional_sbml_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalChild<T> {
        OptionalChild::new(self.xml_element(), name, URL_SBML_CORE)
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the MathML namespace.
    #[inline(always)]
    fn optional_math_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalChild<T> {
        OptionalChild::new(self.xml_element(), name, URL_MATHML)
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the HTML namespace.
    #[inline(always)]
    fn optional_html_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalChild<T> {
        OptionalChild::new(self.xml_element(), name, URL_HTML)
    }

    /// Create an instance of a [RequiredProperty] with the given `name` which adheres to
    /// the SBML namespace.
    #[inline(always)]
    fn required_sbml_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
    ) -> RequiredProperty<T> {
        // TODO: At the moment, properties ignore namespaces.
        RequiredProperty::new(self.xml_element(), name)
    }

    /// Create an instance of a [OptionalProperty] with the given `name` which adheres to
    /// the SBML namespace.
    #[inline(always)]
    fn optional_sbml_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
    ) -> OptionalProperty<T> {
        // TODO: At the moment, properties ignore namespaces.
        OptionalProperty::new(self.xml_element(), name)
    }
}

/// [SbmlUtils] is implemented for all types that implement [SBase].
impl<T: SBase> SbmlUtils for T {}
