// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

use crate::constants::namespaces::{NS_SBML_CORE, URL_HTML, URL_MATHML, URL_SBML_CORE};
use crate::core::validation::{
    matches_sboterm_pattern, matches_sid_pattern, matches_xml_id_pattern,
};
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredProperty, XmlDocument, XmlElement, XmlPropertyType,
    XmlWrapper,
};
use biodivine_xml_doc::{Document, Element};
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SId(String);

impl From<SId> for String {
    fn from(value: SId) -> Self {
        value.0
    }
}

impl TryFrom<String> for SId {
    type Error = String; // This could just be a String with the description of the error.

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Here, we need to validate that value is a valid SBML ID according to rules in the specification.

        if matches_sid_pattern(&Some(value.clone())) {
            Ok(Self(value))
        } else {
            Err(format!("ID '{value}' does not represent a valid SId."))
        }
    }
}

impl XmlPropertyType for SId {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        // value.map(|value| {
        //     SbmlId::try_from(value.to_string())
        // }).transpose()
        match value {
            Some(value) => match SId::try_from(value.to_string()) {
                Ok(id) => Ok(Some(id)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MetaId(String);

impl From<MetaId> for String {
    fn from(value: MetaId) -> Self {
        value.0
    }
}
impl TryFrom<String> for MetaId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if matches_xml_id_pattern(&Some(value.clone())) {
            Ok(Self(value))
        } else {
            Err(format!(
                "MetaId {value} does not represent a valid Meta ID (XML ID)."
            ))
        }
    }
}

impl XmlPropertyType for MetaId {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match MetaId::try_from(value.to_string()) {
                Ok(meta_id) => Ok(Some(meta_id)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SboTerm(String);

impl From<SboTerm> for String {
    fn from(value: SboTerm) -> Self {
        value.0
    }
}

impl TryFrom<String> for SboTerm {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if matches_sboterm_pattern(&Some(value.clone())) {
            Ok(Self(value))
        } else {
            Err(format!(
                "SboTerm '{value}' does not represent a valid SboTerm."
            ))
        }
    }
}

impl XmlPropertyType for SboTerm {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match SboTerm::try_from(value.to_string()) {
                Ok(sbo_term) => Ok(Some(sbo_term)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus, there is no need to implement concrete structure.
pub trait SBase: XmlWrapper {
    fn id(&self) -> OptionalProperty<SId> {
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
                parent = parent.parent(read_doc.deref())?;
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
