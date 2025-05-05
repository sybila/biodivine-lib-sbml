// TODO:
//      At some point, we should probably have an `SId` type instead of using a string here,
//      because IDs have a special format that should be enforced. This is also related to other
//      types that are "string like", e.g. meta id and sboTerm.

use crate::constants::namespaces::{Namespace, NS_SBML_CORE, URL_HTML, URL_MATHML, URL_SBML_CORE};
use crate::core::validation::{
    matches_sboterm_pattern, matches_sid_pattern, matches_xml_id_pattern,
};
use crate::xml::py::runtime_error;
use crate::xml::{
    OptionalChild, OptionalSbmlProperty, RequiredChild, RequiredSbmlProperty, XmlDocument,
    XmlElement, XmlPropertyType, XmlWrapper,
};
use crate::Sbml;
use biodivine_xml_doc::{Document, Element};
use pyo3::{pyclass, pymethods, PyResult};
use sbml_macros::PythonPropertyType;
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PythonPropertyType)]
#[pyclass]
pub struct SId(String);

impl SId {
    /// Utility method to access the underlying string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[pymethods]
impl SId {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Self::try_from(value.to_string()).map_err(runtime_error)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("SId({})", self.0))
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, PythonPropertyType)]
#[pyclass]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, PythonPropertyType)]
#[pyclass]
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

    fn notes(&self) -> OptionalChild<XmlElement> {
        self.optional_sbml_child("notes")
    }

    fn annotation(&self) -> OptionalChild<XmlElement> {
        self.optional_sbml_child("annotation")
    }

    /// Returns the root [`Sbml`] object, assuming the root of the containing document is an
    /// `<sbml>` tag. For detached elements, this uses the internal  [`XmlDocument`] reference
    /// to obtain the document root directly.
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
    ///
    /// Warning: Depending on the specific contract of the underlying type, this can create
    /// an element that is not in a valid state (e.g. missing certain required attributes).
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

    #[inline(always)]
    fn optional_package_child<T: XmlWrapper>(
        &self,
        name: &'static str,
        extension: Namespace,
        required: bool,
    ) -> OptionalChild<T> {
        // TODO:
        //  This should probably create the package declaration only when the element is
        //  written, and check that the package declaration is present if the element is read.
        //  However, for that, we will need to derive a new sub-type from `XmlChild`... -_-

        // TODO 2:
        //  SBML packages are always either required or not required. I.e. the required flag
        //  can be part of the namespace "object" and we don't need to set it dynamically
        //  based on which document elements are accessed.
        self.ensure_package(extension, required);
        OptionalChild::new(self.xml_element(), name, extension.1)
    }

    #[inline(always)]
    fn required_package_child<T: XmlWrapper>(
        &self,
        name: &'static str,
        extension: Namespace,
        required: bool,
    ) -> RequiredChild<T> {
        self.ensure_package(extension, required);
        RequiredChild::new(self.xml_element(), name, extension.1)
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

    /// Ensures the root `<sbml>` tag correctly declares a package namespace.
    fn ensure_package(&self, namespace: Namespace, required: bool) {
        let sbml = self.sbml_root();
        sbml.ensure_sbml_package(namespace, required).unwrap();
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
        unsafe { Some(E::unchecked_cast(e)) }
    }

    fn find_element_by_sid(&self, id: &SId) -> Option<XmlElement> {
        let doc = self.document();
        let doc = doc.read().unwrap();

        let root = doc.root_element()?;

        for child in root.child_elements_recursive(&doc) {
            for (name, value) in child.attributes(&doc) {
                let (_namespace, attr_name) = Element::separate_prefix_name(name.as_str());
                // TODO: Can we check if the namespace corresponds to an SBML extension? Probably not...
                if attr_name == "id" && id.as_str() == value.as_str() {
                    return Some(XmlElement::new_raw(self.document(), child));
                }
            }
        }

        None
    }

    fn find_by_meta_id<E: SBase>(&self, id: &MetaId) -> Option<E> {
        let e = self.find_element_by_meta_id(id)?;
        // TODO: This should check that the conversion can succeed?
        unsafe { Some(E::unchecked_cast(e)) }
    }

    fn find_element_by_meta_id(&self, id: &MetaId) -> Option<XmlElement> {
        let doc = self.document();
        let doc = doc.read().unwrap();

        let root = doc.root_element()?;

        for child in root.child_elements_recursive(&doc) {
            for (name, value) in child.attributes(&doc) {
                let (_namespace, attr_name) = Element::separate_prefix_name(name.as_str());
                // TODO: Can we check if the namespace corresponds to an SBML extension? Probably not...
                if attr_name == "metaid" && id.as_str() == value.as_str() {
                    return Some(XmlElement::new_raw(self.document(), child));
                }
            }
        }

        None
    }
}

/// [crate::sbase::SbmlUtils] is implemented for all types that implement [crate::sbase::SBase].
impl<T: SBase> SbmlUtils for T {}
