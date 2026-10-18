use crate::xml::{XmlElement, XmlPropertyType};
use biodivine_lib_xml_dom::QualifiedName;

/// Resolve the expanded name of an attribute from its written form.
///
/// A name of the form `prefix:local` (for example `xsi:type`) is resolved against the
/// namespace declarations that are in scope for `element`. A name without a prefix is an
/// attribute in no namespace, which is how unqualified attributes are always spelled.
pub(crate) fn resolve_attribute_name(
    element: &XmlElement,
    name: &str,
) -> Result<QualifiedName, String> {
    if let Some((prefix, _local)) = name.split_once(':') {
        if !prefix.is_empty() {
            return element
                .element()
                .resolve_attribute_name(name)
                .map_err(|it| it.to_string());
        }
    }
    QualifiedName::without_namespace(name).map_err(|it| it.to_string())
}

/// Any [XmlProperty] object provides type-safe access to a single XML attribute
/// of an underlying tag.
///
/// Note that implementations of [XmlProperty] do not store the value of the attribute.
/// Instead, [XmlProperty] objects maintain a reference to the underlying [XmlElement] and
/// manipulate this element directly.
///
/// [XmlProperty] is also parametrized by a type `T` which is the underlying "type" of
/// the property. This type `T` must then implement [XmlPropertyType], which is used
/// to facilitate the conversion. However, note that this conversion process only detects
/// "local" data integrity errors (e.g. invalid integer). More "global" checks are handled
/// through a separate document-wide validation.
///
/// In practice, we use two derived "variants" inheriting from [XmlProperty]:
/// [RequiredXmlProperty] and [OptionalXmlProperty]. These provide the two common "default"
/// behaviors for treating missing values.
pub trait XmlProperty<T: XmlPropertyType>: Sized {
    /// Returns a reference to the underlying [XmlElement].
    fn element(&self) -> &XmlElement;

    /// The **expanded** name of the underlying XML attribute.
    ///
    /// An unprefixed SBML core attribute is in *no* namespace (XML attributes never take the
    /// default namespace), so the default implementation simply returns an attribute name
    /// without a namespace. Implementations whose attribute belongs to a non-empty namespace
    /// (an SBML package) override this method; they can also resolve the namespace prefix
    /// dynamically based on the position of the attribute in the document.
    ///
    /// If `write_doc` is `true`, the method is allowed to ensure the conditions necessary for
    /// the returned name to be usable (in particular, to declare a namespace that the document
    /// does not declare yet). Read-only callers pass `false` and must not modify the document.
    fn qualified_name(&self, _write_doc: bool) -> Result<QualifiedName, String> {
        resolve_attribute_name(self.element(), self.simple_name())
    }

    /// Returns the **simple** name of the underlying XML attribute, i.e. excluding any
    /// namespace prefix or similar.
    ///
    /// This name should be associated with the property and be effectively constant.
    fn simple_name(&self) -> &str;

    /// Returns `true` if the underlying XML attribute has a known, set value.
    ///
    /// This refers directly to the value in the underlying XML document, not the value produced
    /// by the [XmlPropertyType] conversion. The conversion can still yield a default value even
    /// if the attribute is missing, or give an error if the value is invalid.
    fn is_set(&self) -> bool {
        let element = self.element();
        let Ok(name) = self.qualified_name(false) else {
            // If the name cannot be built, the property has no value.
            return false;
        };
        element.element().attribute(&name).is_some()
    }

    /// Read the value of this [XmlProperty], or a `String` error if the underlying value
    /// is invalid. The function can return `None` if the attribute is missing, or if an
    /// equivalent to the `None` value is written in the document.
    ///
    ///  > See [XmlPropertyType] for constraints on the error format and general notes about
    ///  > value conversion.
    fn get_checked(&self) -> Result<Option<T>, String> {
        let element = self.element();
        let name = self.qualified_name(false)?;
        let raw = element.element().attribute(&name);
        let raw = raw.as_ref().map(|it| it.as_str());
        // See [XmlPropertyType::trims_whitespace]: a value that uses an XML Schema datatype with
        // the "collapse" whitespace facet may carry incidental padding in the document.
        let raw = if T::trims_whitespace() {
            raw.map(str::trim)
        } else {
            raw
        };
        XmlPropertyType::try_get(raw)
    }

    /// Read the "raw" underlying attribute value of this [XmlProperty], or `None` if the value
    /// is not set.
    fn get_raw(&self) -> Option<String> {
        let element = self.element();
        let Ok(name) = self.qualified_name(false) else {
            // If the name cannot be built, the property has no value.
            return None;
        };
        element
            .element()
            .attribute(&name)
            .map(|it| it.as_str().to_string())
    }

    /// Remove the underlying XML attribute completely.
    ///
    /// # Document validity
    ///
    /// This function can make the underlying property *invalid* if a missing attribute
    /// does not map to any valid property value.
    fn clear(&self) -> Result<(), String> {
        let element = self.element();
        let name = self.qualified_name(true)?;
        element.element().remove_attribute(&name);
        Ok(())
    }

    /// Write a raw `value` into this [XmlProperty].
    ///
    /// # Document validity
    ///
    /// Obviously, this function can be used to set the property to a completely invalid value.
    fn set_raw(&self, value: String) -> Result<(), String> {
        let element = self.element();
        let name = self.qualified_name(true)?;
        if let Some(namespace) = name.namespace() {
            // Make sure the namespace is in scope, so that the resulting document is valid.
            element.ensure_namespace(namespace);
        }
        element
            .element()
            .set_attribute_checked(name, value)
            .map_err(|it| it.to_string())
    }
}

/// A variant of [XmlProperty] that covers a property that can be missing in a valid document.
pub trait OptionalXmlProperty<T: XmlPropertyType>: XmlProperty<T> {
    /// Read the value of an optional XML property.
    ///
    /// # Panics
    ///
    /// Panics if the [XmlProperty::get_checked] produces an error.
    fn get(&self) -> Option<T> {
        match self.get_checked() {
            Err(error) => panic!(
                "Invalid value for attribute `{}`: {}",
                self.simple_name(),
                error
            ),
            Ok(value) => value,
        }
    }

    /// Write the value of an optional XML property.
    ///
    /// # Panics
    ///
    /// Panics if [XmlProperty::set_raw] produces an error (typically due to namespace issues).
    fn set(&self, value: Option<&T>) {
        match value.and_then(|it| it.set()) {
            None => self.clear(),
            Some(value) => self.set_raw(value),
        }
        .unwrap()
    }

    /// An alternative to [OptionalXmlProperty::set] that accepts a value directly, without
    /// wrapping it into `Option`.
    ///
    /// # Panics
    ///
    /// Panics if the [XmlProperty::clear] or [XmlProperty::set_raw] method produces an error
    /// (typically due to namespace issues).
    fn set_some(&self, value: &T) {
        match value.set() {
            None => self.clear(),
            Some(value) => self.set_raw(value),
        }
        .unwrap()
    }
}

/// A variant of [XmlProperty] that covers a property that is required to have a value in
/// a valid document.
///
/// Note that this value can be a default value, in which case the property may not actually be
/// set in the document; it merely has a default value.
pub trait RequiredXmlProperty<T: XmlPropertyType>: XmlProperty<T> {
    /// Read the value of a required XML property.
    ///
    /// # Panics
    ///
    /// Panics if the [XmlProperty::get_checked] method produces an error or a `None` value.
    fn get(&self) -> T {
        match self.get_checked() {
            Err(error) => panic!(
                "Invalid value for attribute `{}`: {}",
                self.simple_name(),
                error
            ),
            Ok(None) => {
                panic!("Missing value for attribute `{}`.", self.simple_name())
            }
            Ok(Some(value)) => value,
        }
    }

    /// Write the value of a required XML property.
    ///
    /// Note that the method can actually erase the XML attribute if the written value represents
    /// the "default" value for this type, and it can be correctly represented by
    /// a missing attribute.
    ///
    /// # Panics
    ///
    /// Panics if [XmlProperty::set_raw] or [XmlProperty::clear] method produces an error,
    /// typically due to namespace issues.
    fn set(&self, value: &T) {
        match value.set() {
            None => self.clear(),
            Some(value) => self.set_raw(value),
        }
        .unwrap();
    }
}
