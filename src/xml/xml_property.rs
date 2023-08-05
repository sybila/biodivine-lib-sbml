use crate::xml::{XmlElement, XmlPropertyType};

/// Any [XmlProperty] object provides type-safe access to a single XML attribute
/// of an underlying tag.
///
/// [XmlProperty] objects typically maintain a reference to the underlying [XmlElement]
/// and thus cannot be stored or passed around independently.
///
/// [XmlProperty] is also parametrized by a type `T` which is the underlying "type" of
/// the property. This type `T` must then implement [XmlPropertyType], which is used
/// to facilitate the conversion. There could be safety checks that cannot be performed by
/// [XmlPropertyType] directly. These should be performed separately by the underlying
/// [XmlElement]/[XmlDocument] (e.g. if an ID is required to be unique in the whole document).
///
/// TODO: Document the derive macro.
///
/// ## On missing attributes and value validity
///
/// Note that whether a missing value is considered valid is implementation specific and
/// depends on `T`. I.e. there can be types that always *require* some value, while there
/// can be types where a missing value represents some sort of default.
///
/// In general, it is recommended that when a missing value is considered valid but there
/// is no suitable `T::default()` value, one should use `T = Option<R>` (i.e. `T` is an
/// optional type). If there is a `T::default()` value, it is possible to return this value
/// when the attribute value is missing.
///
/// Similarly, when writing a value, if the property is optional (e.g. `T = Option<R>`), then
/// write functions are allowed to erase the attribute if `None` is being written, assuming there
/// is no other appropriate value that represents `None`.
pub trait XmlProperty<T: XmlPropertyType>: Sized {
    /// Returns a reference to the underlying [XmlElement].
    fn element(&self) -> &XmlElement;

    /// Returns `true` if the underlying XML attribute has a known set value, even if such
    /// value is invalid.
    ///
    /// Note that this refers directly to value in the underlying document. When the attribute
    /// value is missing, this function must return `false`, even if a missing value
    /// is valid for type `T`.
    fn is_set(&self) -> bool;

    /// Returns `true` if the underlying XML attribute represents a valid value of type `T`.
    ///
    /// See the overall discussion in [XmlProperty] regarding how to treat validity of missing
    /// attribute values.
    fn is_valid(&self) -> bool {
        self.read_checked().is_ok()
    }

    /// Read the value of this [XmlProperty].
    ///
    /// # Panics
    ///
    /// The function should panic if the underlying attribute value is invalid for type `T`.
    fn read(&self) -> T;

    /// Read the value of this [XmlProperty], or a `String` error if the underlying value
    /// is invalid.
    ///
    ///  > The `String` error should be a full English sentence (or sentences). It should contain
    ///  the name of the XML attribute, it's current value, and the reason why the value is invalid.
    ///
    /// See the overall discussion in [XmlProperty] regarding how to treat validity of missing
    /// attribute values.
    fn read_checked(&self) -> Result<T, String>;

    /// Read the "raw" underlying attribute value of this [XmlProperty], or `None` if the value
    /// is not set.
    fn read_raw(&self) -> Option<String>;

    /// Remove the underlying XML attribute completely.
    ///
    /// # Safety
    ///
    /// Note that this function can make the underlying property *invalid* if a missing attribute
    /// does not map to any valid property value.
    fn clear(&self);

    /// Write given [value] into this [XmlProperty].
    ///
    /// See the overall discussion in [XmlProperty] regarding how to treat missing/default
    /// attribute values.
    fn write(&self, value: &T);

    /// Write a raw [value] into this [XmlProperty].
    ///
    /// # Safety
    ///
    /// Obviously, this function can be used to set the property to an invalid value.
    fn write_raw(&self, value: String);
}
