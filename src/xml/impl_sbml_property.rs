use crate::constants::namespaces::{Namespace, URL_SBML_CORE};
use crate::xml::{XmlElement, XmlPropertyType, XmlWrapper};
use crate::Sbml;
use std::marker::PhantomData;
use std::ops::Deref;

/// A variant of [crate::xml::Property] which adheres to SBML package prefix resolution rules.
///
/// The rules are the following:
///  - An SBML property must be declared with a namespace URL (either the core namespace, or
///    an SBML package namespace).
///  - If the property is in the core namespace, it *must* be defined without any prefix, even
///    if the core namespace itself is defined with a prefix.
///  - If the property is in a package namespace, the behavior is different depending on whether
///    the element is the same package or different (in the core specification, this only applies
///    to the core package, but it makes sense for all instances where one
///    package extends another):
///     * For elements from different packages (or core), the property *must* be used
///       with the namespace prefix.
///     * For same package elements, the property *can* be used with the namespace prefix, or
///       without a prefix (either is fine). If both (or none) are declared, the one *with*
///       namespace prefix is preferred, as it appears to have better backwards compatibility
///       (this last part is not officially in the SBML specification).
pub struct SbmlProperty<'a, T: XmlPropertyType> {
    element: &'a XmlElement,
    name: &'static str,
    property_namespace: Namespace,
    element_namespace: Namespace,
    _marker: PhantomData<T>,
}

/// A variant of [SbmlProperty] where the value of the property can be `None` (i.e. unset).
pub struct OptionalSbmlProperty<'a, T: XmlPropertyType>(SbmlProperty<'a, T>);
/// A variant of [SbmlProperty] where it is invalid for the value of the property to be missing.
pub struct RequiredSbmlProperty<'a, T: XmlPropertyType>(SbmlProperty<'a, T>);

impl<'a, T: XmlPropertyType> SbmlProperty<'a, T> {
    /// Create a new instance of a [SbmlProperty] for the given `element`, `name` and `package_url`.
    pub fn new(
        element: &'a XmlElement,
        name: &'static str,
        property_package_url: Namespace,
        element_package_url: Namespace,
    ) -> SbmlProperty<'a, T> {
        SbmlProperty {
            element,
            name,
            property_namespace: property_package_url,
            element_namespace: element_package_url,
            _marker: Default::default(),
        }
    }

    /// Check if this property belongs to the SBML core namespace.
    pub fn is_sbml_core(&self) -> bool {
        // If we ever need to support more than one SBML version, this will need to be updated.
        self.property_namespace.1 == URL_SBML_CORE
    }

    /// Check if the (self-reported) element of this property is in the same SBML package as the
    /// property itself.
    pub fn is_the_same_package_as_element(&self) -> bool {
        self.property_namespace == self.element_namespace
    }
}

impl<'a, T: XmlPropertyType> OptionalSbmlProperty<'a, T> {
    pub fn new(
        element: &'a XmlElement,
        name: &'static str,
        property_namespace: Namespace,
        element_namespace: Namespace,
    ) -> OptionalSbmlProperty<'a, T> {
        OptionalSbmlProperty(SbmlProperty::new(
            element,
            name,
            property_namespace,
            element_namespace,
        ))
    }
}

impl<'a, T: XmlPropertyType> RequiredSbmlProperty<'a, T> {
    pub fn new(
        element: &'a XmlElement,
        name: &'static str,
        property_namespace: Namespace,
        element_namespace: Namespace,
    ) -> RequiredSbmlProperty<'a, T> {
        RequiredSbmlProperty(SbmlProperty::new(
            element,
            name,
            property_namespace,
            element_namespace,
        ))
    }
}

impl<T: XmlPropertyType> crate::xml::XmlProperty<T> for SbmlProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.element
    }

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        if self.is_sbml_core() {
            // If the property is in the core namespace, it must be used without a prefix, using
            // just its name. No need to manipulate any package declarations.
            Ok(self.name.to_string())
        } else {
            let Some(sbml_root) = Sbml::try_for_child_element(self.element) else {
                return Err(format!(
                    "SBML property `{}` used outside of an SBML document.",
                    self.name
                ));
            };

            let prefix = if write_doc {
                sbml_root.ensure_sbml_package(self.property_namespace, false)?
            } else {
                sbml_root.find_sbml_package(self.property_namespace)?
            };

            if !self.is_the_same_package_as_element() {
                // In an element from a different package (or core), a package property must
                // always be declared using a prefix.
                Ok(format!("{}:{}", prefix, self.name))
            } else {
                // In the same package element, a package property can be defined using a prefix
                // (which still *must* exist anyway), but it can also be defined without a prefix.
                // The preferred option is to use the prefix, but only if the attribute without a
                // prefix does not already exist.
                let doc = self.element.read_doc();
                let attributes = self.element.element.attributes(doc.deref());
                if attributes.contains_key(self.simple_name()) {
                    // If the attribute without prefix already exists, we use it directly.
                    Ok(self.simple_name().to_string())
                } else {
                    // If the attribute without prefix does not exist, we use the fully quantified
                    // name *with* the prefix.
                    Ok(format!("{}:{}", prefix, self.name))
                }
            }
        }
    }

    fn simple_name(&self) -> &str {
        self.name
    }
}

impl<T: XmlPropertyType> crate::xml::XmlProperty<T> for OptionalSbmlProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        self.0.quantified_name(write_doc)
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }
}

impl<T: XmlPropertyType> crate::xml::XmlProperty<T> for RequiredSbmlProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        self.0.quantified_name(write_doc)
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }
}

impl<T: XmlPropertyType> crate::xml::OptionalXmlProperty<T> for OptionalSbmlProperty<'_, T> {}
impl<T: XmlPropertyType> crate::xml::RequiredXmlProperty<T> for RequiredSbmlProperty<'_, T> {}
