use crate::xml::{XmlElement, XmlPropertyType, XmlWrapper};
use std::ops::{Deref, DerefMut};

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
/// behaviours for treating missing values.
///
pub trait XmlProperty<T: XmlPropertyType>: Sized {
    /// Returns a reference to the underlying [XmlElement].
    fn element(&self) -> &XmlElement;

    /// Returns the **fully quantified** name of the underlying XML attribute, including
    /// namespace prefix if relevant. Can return an error at runtime if there is some problem
    /// with the construction of the quantified name (e.g. the namespace is not declared, or
    /// not declared correctly).
    ///
    /// This name can (and probably should be) computed dynamically at runtime for properties
    /// that belong to a specific non-default namespace, as the prefix can change depending
    /// on the position of the property in the document.
    ///
    /// If `write_doc` is set to `true`, it indicates to the method that it can try to
    /// ensure necessary conditions for the quantified name to be valid (e.g. create a namespace
    /// declaration). The exact conditions as to when this is valid can vary depending on the
    /// implementation. In general, methods that only read the document should not allow
    /// any modification. Meanwhile, methods that write values to the document can set this to
    /// true in order to indicate that "fixing" the document into a consistent state is allowed.
    ///
    /// The default implementation for this method simply returns [XmlProperty::simple_name]
    /// (i.e. it assumes the attribute is in the default empty namespace). Please override this
    /// in cases where the property can depend on XML namespaces.
    fn quantified_name(&self, _write_doc: bool) -> Result<String, String> {
        Ok(self.simple_name().to_string())
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
        let Ok(name) = self.quantified_name(false) else {
            // If the quantified name can't be built, the property has no value.
            return false;
        };
        // As opposed to `self.read_raw().is_some()`, this does not need to copy the attribute.
        let doc = element.read_doc();
        element
            .raw_element()
            .attribute(doc.deref(), name.as_str())
            .is_some()
    }

    /// Read the value of this [XmlProperty], or a `String` error if the underlying value
    /// is invalid. The function can return `None` if the attribute is missing, or if an
    /// equivalent to the `None` value is written in the document.
    ///
    ///  > See [XmlPropertyType] for constraints on the error format and general notes about
    ///  > value conversion.
    fn get_checked(&self) -> Result<Option<T>, String> {
        let element = self.element();
        let name = self.quantified_name(false)?;
        let doc = element.read_doc();
        let value = element.raw_element().attribute(doc.deref(), name.as_str());
        XmlPropertyType::try_get(value)
    }

    /// Read the "raw" underlying attribute value of this [XmlProperty], or `None` if the value
    /// is not set.
    fn get_raw(&self) -> Option<String> {
        let element = self.element();
        let Ok(name) = self.quantified_name(false) else {
            // If the quantified name can't be built, the property has no value.
            return None;
        };
        let doc = element.read_doc();
        element
            .raw_element()
            .attribute(doc.deref(), name.as_str())
            .map(|it| it.to_string())
    }

    /// Remove the underlying XML attribute completely.
    ///
    /// # Document validity
    ///
    /// This function can make the underlying property *invalid* if a missing attribute
    /// does not map to any valid property value.
    fn clear(&self) -> Result<(), String> {
        let element = self.element();
        let name = self.quantified_name(true)?;
        let mut doc = element.write_doc();
        element
            .raw_element()
            .mut_attributes(doc.deref_mut())
            .remove(name.as_str());
        Ok(())
    }

    /// Write a raw `value` into this [XmlProperty].
    ///
    /// # Document validity
    ///
    /// Obviously, this function can be used to set the property to a completely invalid value.
    fn set_raw(&self, value: String) -> Result<(), String> {
        let element = self.element();
        let name = self.quantified_name(true)?;
        let mut doc = element.write_doc();
        element
            .raw_element()
            .set_attribute(doc.deref_mut(), name, value);
        Ok(())
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
