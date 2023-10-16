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

    /// Returns the name of the underlying XML attribute.
    fn name(&self) -> &str;

    /// Returns `true` if the underlying XML attribute has a known, set value.
    ///
    /// This refers directly to the value in the underlying XML document, not the value produced
    /// by the [XmlPropertyType] conversion. The conversion can still yield a default value even
    /// if the attribute is missing, or give an error if the value is invalid.
    fn is_set(&self) -> bool {
        let element = self.element();
        let name = self.name();
        // As opposed to `self.read_raw().is_some()`, this does not need to copy the attribute.
        let doc = element.read_doc();
        element.raw_element().attribute(doc.deref(), name).is_some()
    }

    /// Read the value of this [XmlProperty], or a `String` error if the underlying value
    /// is invalid. The function can return `None` if the attribute is missing, or if an
    /// equivalent to the `None` value is written in the document.
    ///
    ///  > See [XmlPropertyType] for constraints on the error format and general notes about
    ///  > value conversion.
    fn read_checked(&self) -> Result<Option<T>, String> {
        let element = self.element();
        let name = self.name();
        let doc = element.read_doc();
        let value = element.raw_element().attribute(doc.deref(), name);
        XmlPropertyType::try_read(value)
    }

    /// Read the "raw" underlying attribute value of this [XmlProperty], or `None` if the value
    /// is not set.
    fn read_raw(&self) -> Option<String> {
        let element = self.element();
        let name = self.name();
        let doc = element.read_doc();
        element
            .raw_element()
            .attribute(doc.deref(), name)
            .map(|it| it.to_string())
    }

    /// Remove the underlying XML attribute completely.
    ///
    /// # Document validity
    ///
    /// This function can make the underlying property *invalid* if a missing attribute
    /// does not map to any valid property value.
    fn clear(&self) {
        let element = self.element();
        let name = self.name();
        let mut doc = element.write_doc();
        element
            .raw_element()
            .mut_attributes(doc.deref_mut())
            .remove(name);
    }

    /// Write a raw [value] into this [XmlProperty].
    ///
    /// # Document validity
    ///
    /// Obviously, this function can be used to set the property to a completely invalid value.
    fn write_raw(&self, value: String) {
        let element = self.element();
        let name = self.name();
        let mut doc = element.write_doc();
        element
            .raw_element()
            .set_attribute(doc.deref_mut(), name, value);
    }
}

/// A variant of [XmlProperty] that covers a property that can be missing in a valid document.
pub trait OptionalXmlProperty<T: XmlPropertyType>: XmlProperty<T> {
    /// Read the value of an optional XML property.
    ///
    /// # Panics
    ///
    /// Panics if the [XmlProperty::read_checked] produces an error.
    fn read(&self) -> Option<T> {
        match self.read_checked() {
            Err(error) => panic!("Invalid value for attribute `{}`: {}", self.name(), error),
            Ok(value) => value,
        }
    }

    /// Write the value of an optional XML property.
    ///
    /// TODO: I'm not sure whether `Option<&T>` or `&Option<T>` is better here. The time will tell.
    fn write(&self, value: Option<&T>) {
        match value.and_then(|it| it.write()) {
            None => self.clear(),
            Some(value) => self.write_raw(value),
        }
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
    /// Panics if the [XmlProperty::read_checked] method produces an error or a `None` value.
    fn read(&self) -> T {
        match self.read_checked() {
            Err(error) => panic!("Invalid value for attribute `{}`: {}", self.name(), error),
            Ok(None) => panic!("Missing value for attribute `{}`.", self.name()),
            Ok(Some(value)) => value,
        }
    }

    /// Write the value of a required XML property.
    ///
    /// Note that the method can actually erase the XML attribute if the written value represents
    /// the "default" value for this type, and it can be correctly represented by
    /// a missing attribute.
    fn write(&self, value: &T) {
        match value.write() {
            None => self.clear(),
            Some(value) => self.write_raw(value),
        };
    }
}
