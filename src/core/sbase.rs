// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

use crate::constants::namespaces::{Namespace, NS_HTML, NS_MATHML, NS_SBML_CORE, URL_SBML_CORE};
use crate::core::validation::{
    matches_sboterm_pattern, matches_sid_pattern, matches_xml_id_pattern,
};
use crate::xml::{
    Document, OptionalChild, OptionalSbmlChild, OptionalSbmlProperty, RequiredSbmlChild,
    RequiredSbmlProperty, XmlElement, XmlPropertyType, XmlWrapper,
};
use crate::Sbml;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SId(String);

impl SId {
    /// Utility method to access the underlying string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for SId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SId> for String {
    fn from(value: SId) -> Self {
        value.0
    }
}

impl TryFrom<String> for SId {
    type Error = String; // This could just be a String with the description of the error.

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Here, we need to validate that value is a valid SBML ID according to rules in the specification.
        // TODO:
        //      `matches_sid_pattern` is not a very good API, because we need to copy the
        //      string value here, but that's unfortunately a past mistake that's not important
        //      right now.
        if matches_sid_pattern(&Some(value.clone())) {
            Ok(Self(value))
        } else {
            Err(format!("ID '{value}' does not represent a valid SId."))
        }
    }
}

impl TryFrom<&str> for SId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_string();
        SId::try_from(value)
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

impl MetaId {
    /// Utility method to access the underlying string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for MetaId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl TryFrom<&str> for MetaId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_string();
        MetaId::try_from(value)
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

impl SboTerm {
    /// Utility method to access the underlying string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for SboTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl TryFrom<&str> for SboTerm {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_string();

        SboTerm::try_from(value)
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
    fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("id")
    }

    fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_sbml_property("name")
    }

    fn meta_id(&self) -> OptionalSbmlProperty<MetaId> {
        self.optional_sbml_property("metaid")
    }

    fn sbo_term(&self) -> OptionalSbmlProperty<SboTerm> {
        self.optional_sbml_property("sboTerm")
    }

    fn notes(&self) -> OptionalSbmlChild<XmlElement> {
        self.optional_sbml_child("notes")
    }

    fn annotation(&self) -> OptionalSbmlChild<XmlElement> {
        self.optional_sbml_child("annotation")
    }

    /// Returns the root [`Sbml`] object, assuming the root of the containing document is an
    /// `<sbml>` tag. For detached elements, this uses the internal [`Document`] reference
    /// to get the document root directly.
    ///
    /// The method panics if the element is not a member of an SBML document.
    fn sbml_root(&self) -> Sbml {
        Sbml::try_for_child(self).unwrap()
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
        let mut parent = child.element().clone();
        loop {
            if parent.local_name().as_str() == tag_name
                && parent
                    .namespace()
                    .is_some_and(|it| it.uri() == URL_SBML_CORE)
            {
                break;
            }
            parent = parent.parent()?.as_element()?;
        }
        Some(Self::from(XmlElement::new(parent)))
    }

    /// Create a new instance of `Self` which is just an empty tag with the given `tag_name`
    /// and using SBML namespace.
    ///
    /// This is the *named* form of the creation path: it is what a supertype uses when the element
    /// name is chosen by its caller (a rule is created as an `algebraicRule`, an `assignmentRule` or
    /// a `rateRule`), which is exactly the case in which the type cannot have an identity-derived
    /// constructor (see [`crate::xml::SbmlObject::CONSTRUCT`]). Every other type is created through
    /// its own inherent `new_empty(document)`, which takes the name and the namespace from the
    /// declaration instead of a literal.
    ///
    /// Warning: Depending on the specific contract of the underlying type, this can create
    /// an element that is not in a valid state (e.g., missing certain required attributes).
    #[inline(always)]
    fn new_empty_named(document: Document, tag_name: &str) -> Self {
        let element = XmlElement::new_qualified(document, tag_name, NS_SBML_CORE);
        Self::from(element)
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the SBML namespace.
    #[inline(always)]
    fn optional_sbml_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalSbmlChild<T> {
        OptionalSbmlChild::new(self.xml_element(), name, NS_SBML_CORE)
    }

    #[inline(always)]
    fn optional_package_child<T: XmlWrapper>(
        &self,
        name: &'static str,
        extension: Namespace,
        _required: bool,
    ) -> OptionalSbmlChild<T> {
        OptionalSbmlChild::new(self.xml_element(), name, extension)
    }

    #[inline(always)]
    fn required_package_child<T: XmlWrapper>(
        &self,
        name: &'static str,
        extension: Namespace,
        _required: bool,
    ) -> RequiredSbmlChild<T> {
        RequiredSbmlChild::new(self.xml_element(), name, extension)
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the MathML namespace.
    #[inline(always)]
    fn optional_math_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalChild<T> {
        OptionalChild::new(self.xml_element(), name, NS_MATHML)
    }

    /// Create an instance of [OptionalChild] with the given `name` and using the HTML namespace.
    #[inline(always)]
    fn optional_html_child<T: XmlWrapper>(&self, name: &'static str) -> OptionalChild<T> {
        OptionalChild::new(self.xml_element(), name, NS_HTML)
    }

    /// Create an instance of a [RequiredProperty] with the given `name` which adheres to
    /// the SBML namespace.
    #[inline(always)]
    fn required_sbml_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
    ) -> RequiredSbmlProperty<T> {
        RequiredSbmlProperty::new(self.xml_element(), name, NS_SBML_CORE, NS_SBML_CORE)
    }

    /// Create an instance of a [OptionalProperty] with the given `name` which adheres to
    /// the SBML namespace.
    #[inline(always)]
    fn optional_sbml_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
    ) -> OptionalSbmlProperty<T> {
        OptionalSbmlProperty::new(self.xml_element(), name, NS_SBML_CORE, NS_SBML_CORE)
    }

    fn optional_package_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
        property_package: Namespace,
        element_package: Namespace,
    ) -> OptionalSbmlProperty<T> {
        OptionalSbmlProperty::new(self.xml_element(), name, property_package, element_package)
    }

    fn required_package_property<T: XmlPropertyType>(
        &self,
        name: &'static str,
        property_package: Namespace,
        element_package: Namespace,
    ) -> RequiredSbmlProperty<T> {
        RequiredSbmlProperty::new(self.xml_element(), name, property_package, element_package)
    }

    fn find_by_sid<E: SBase>(&self, id: &SId) -> Option<E> {
        let e = self.find_element_by_sid(id)?;
        // TODO: This should check that the conversion can succeed?
        Some(E::from(e))
    }

    fn find_element_by_sid(&self, id: &SId) -> Option<XmlElement> {
        let root = self.xml_element().document().root()?;

        for child in root.descendants() {
            let Some(child) = child.as_element() else {
                continue;
            };
            for (name, value) in child.attributes() {
                // TODO: Can we check if the namespace corresponds to an SBML extension? Probably not...
                if name.local_name_str() == "id" && id.as_str() == value.as_str() {
                    return Some(XmlElement::new(child));
                }
            }
        }

        None
    }

    fn find_by_meta_id<E: SBase>(&self, id: &MetaId) -> Option<E> {
        let e = self.find_element_by_meta_id(id)?;
        // TODO: This should check that the conversion can succeed?
        Some(E::from(e))
    }

    fn find_element_by_meta_id(&self, id: &MetaId) -> Option<XmlElement> {
        let root = self.xml_element().document().root()?;

        for child in root.descendants() {
            let Some(child) = child.as_element() else {
                continue;
            };
            for (name, value) in child.attributes() {
                // TODO: Can we check if the namespace corresponds to an SBML extension? Probably not...
                if name.local_name_str() == "metaid" && id.as_str() == value.as_str() {
                    return Some(XmlElement::new(child));
                }
            }
        }

        None
    }
}

/// [crate::sbase::SbmlUtils] is implemented for all types that implement [crate::sbase::SBase].
impl<T: SBase> SbmlUtils for T {}
