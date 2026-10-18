use crate::constants::namespaces::{qualified_name, Namespace, URL_SBML_CORE};
use crate::xml::{XmlElement, XmlProperty, XmlPropertyType};
use crate::Sbml;
use biodivine_lib_xml_dom::QualifiedName;
use std::marker::PhantomData;

/// A variant of [crate::xml::Property] which adheres to SBML package prefix resolution rules.
///
/// The rules are the following:
///  - An SBML property must be declared with a namespace URL (either the core namespace, or
///    an SBML package namespace).
///  - If the property is in the core namespace, it *must* be defined without any prefix, even
///    if the core namespace itself is defined with a prefix. In XML terms, an SBML core
///    attribute lives in *no* namespace, because attributes never take the default namespace.
///  - If the property is in a package namespace, the behaviour depends on whether the element
///    is in the same package or not (in the core specification, this only applies to the core
///    package, but it makes sense for all instances where one package extends another):
///     * For elements from different packages (or core), the property *must* be used
///       with the namespace prefix.
///     * For same package elements, the property *can* be used with the namespace prefix, or
///       without a prefix (either is fine). If both (or none) are declared, the one *with*
///       the namespace prefix is preferred. This last preference is a convention of this crate;
///       the SBML specification does not mandate it.
///
/// Reading is namespace-aware, so the prefix that a document binds to the package URL does not
/// matter; writing always uses the conventional prefix of the package, which the declaration
/// machinery of this module introduces on the `<sbml>` element.
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
    /// Create a new instance of a [SbmlProperty] for the given `element`, `name`,
    /// `property_namespace` and `element_namespace`.
    /// Creates a package-aware accessor for the attribute `name` of `element`.
    pub fn new(
        element: &'a XmlElement,
        name: &'static str,
        property_namespace: Namespace,
        element_namespace: Namespace,
    ) -> SbmlProperty<'a, T> {
        SbmlProperty {
            element,
            name,
            property_namespace,
            element_namespace,
            _marker: PhantomData,
        }
    }

    /// Check if this property belongs to the SBML core namespace.
    pub fn is_sbml_core(&self) -> bool {
        self.property_namespace.1 == URL_SBML_CORE
    }

    /// Check if the (self-reported) element of this property is in the same SBML package as the
    /// property itself.
    pub fn is_the_same_package_as_element(&self) -> bool {
        self.property_namespace == self.element_namespace
    }
}

impl<'a, T: XmlPropertyType> OptionalSbmlProperty<'a, T> {
    /// Creates a package-aware accessor for an attribute that may be absent.
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
    /// Creates a package-aware accessor for an attribute the specification requires.
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        if self.is_sbml_core() {
            // A core attribute is unprefixed and lives in no namespace, so it needs no
            // namespace resolution at all.
            return crate::xml::xml_property::resolve_attribute_name(self.element, self.name);
        }

        // A property whose name is already written as `prefix:local` (for example `xsi:type`)
        // is resolved directly against the namespace declarations that are in scope.
        if self
            .name
            .split_once(':')
            .is_some_and(|(prefix, _)| !prefix.is_empty())
        {
            return crate::xml::xml_property::resolve_attribute_name(self.element, self.name);
        }

        let Some(sbml_root) = Sbml::try_for_child_element(self.element) else {
            return Err(format!(
                "SBML property `{}` used outside of an SBML document.",
                self.name
            ));
        };

        if write_doc {
            // A written name has to be usable: its prefix must be in scope for the whole document,
            // which is what declaring the package on the `<sbml>` element achieves. Reading needs
            // no declaration, because the lookup below compares namespaces, not prefixes.
            sbml_root.activate_package(self.property_namespace)?;
        }

        // A read follows the *document's* version of the package: the accessors are written against
        // the URL this crate writes with, and a Version 1 or Version 3 FBC document keeps the same
        // attribute in a different URL, so a lookup in the written URL would find nothing and
        // report a present attribute as missing. A write keeps the crate's own URL (the property
        // namespace), because a written document is this crate's output.
        let resolved = if write_doc {
            self.property_namespace
        } else {
            crate::constants::namespaces::document_package_namespace_for(
                self.element.element(),
                self.property_namespace,
            )
        };
        let prefixed = qualified_name(resolved, self.name)?;

        if !self.is_the_same_package_as_element() {
            // In an element from a different package (or core), a package property must
            // always be declared using a prefix.
            Ok(prefixed)
        } else {
            // In the same package element, a package property can be defined using a prefix,
            // but it can also be defined without a prefix. The preferred option is to use the
            // prefix, but only if the attribute without a prefix does not already exist.
            let unprefixed =
                QualifiedName::without_namespace(self.name).map_err(|it| it.to_string())?;
            if self.element.element().attribute(&unprefixed).is_some() {
                Ok(unprefixed)
            } else {
                Ok(prefixed)
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }
}

impl<T: XmlPropertyType> crate::xml::XmlProperty<T> for RequiredSbmlProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }
}

impl<T: XmlPropertyType> crate::xml::OptionalXmlProperty<T> for OptionalSbmlProperty<'_, T> {}
impl<T: XmlPropertyType> crate::xml::RequiredXmlProperty<T> for RequiredSbmlProperty<'_, T> {
    /// Reads the value, falling back to the *default* the document's version gives the attribute.
    ///
    /// The crate's typed views are shared between the levels, and the levels disagree about which
    /// attributes are required: `species@constant` is required by Level 3 and optional with the
    /// default `false` in Level 2. A read of a required property therefore cannot be a plain
    /// "the attribute is missing" panic - that would make a *valid* Level 2 document unusable for
    /// every caller that follows the documented read. When the document's level and version give
    /// the attribute a default (`constants::deltas::ATTRIBUTE_DEFAULTS`, one entry per schema
    /// declaration), the read answers with it; when they do not, the documented panic stands,
    /// because then the attribute really is missing for the version the document declares.
    fn get(&self) -> T {
        if let Ok(Some(value)) = self.get_checked() {
            return value;
        }
        let element = self.element().tag_name();
        let name = self.simple_name();
        let version = crate::constants::version::document_level_version_or_base(self.element());
        if let Some(default) = crate::constants::deltas::attribute_default(&element, name, version)
        {
            if let Ok(Some(value)) = T::try_get(Some(default)) {
                return value;
            }
        }
        // The same messages the default implementation produces, so that a caller which does not
        // read the specification's defaults sees exactly the diagnostics it saw before.
        match self.get_checked() {
            Err(error) => panic!("Invalid value for attribute `{}`: {}", name, error),
            Ok(_) => panic!("Missing value for attribute `{}`.", name),
        }
    }
}
