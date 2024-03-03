// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

use crate::constants::namespaces::{NS_SBML_CORE, URL_HTML, URL_MATHML, URL_SBML_CORE};
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredProperty, XmlDocument, XmlElement, XmlPropertyType,
    XmlWrapper,
};
use std::ops::Deref;
use xml_doc::{Document, Element};

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus, there is no need to implement concrete structure.
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
    /// Create a new instance of `Self` by traversing the parents of the given
    /// XML element until the appropriate tag is discovered. If no such tag is found,
    /// returns `None`.
    ///
    /// TODO: Currently, this requires SBML core namespace.
    fn search_in_parents(child: &XmlElement, tag_name: &str) -> Option<Self> {
        let parent = {
            let read_doc = child.read_doc();
            fn check_name(doc: &Document, e: Element, tag_name: &str) -> bool {
                let name = e.name(doc);
                let Some(namespace) = e.namespace(doc) else {
                    return false;
                };

                name == tag_name && namespace == URL_SBML_CORE
            }

            let mut parent = child.raw_element();
            while !check_name(read_doc.deref(), parent, tag_name) {
                let Some(node) = parent.parent(read_doc.deref()) else {
                    return None;
                };
                parent = node;
            }
            parent
        };
        let model = XmlElement::new_raw(child.document(), parent);
        // Safe because we checked that the element has the correct tag name and namespace.
        Some(unsafe { Self::unchecked_cast(model) })
    }

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

/// [crate::sbase::SbmlUtils] is implemented for all types that implement [crate::sbase::SBase].
impl<T: SBase> SbmlUtils for T {}
