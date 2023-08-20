/// [XmlPropertyType] is implemented by types that can be converted to/from an XML attribute.
///
/// Ideally, this conversion should only depend on the value of the XML attribute. I.e. it should
/// not check consistency with the rest of the document. There should be other mechanisms that
/// implement additional checks where necessary, but this should not be included in the "read
/// a value" portion of the process.
///
///  > Technically, this is almost the same as implementing the `TryFrom` and `Into` traits, but
///  should give us a bit more flexibility and clarity regarding what conversions happen where
///  and what error messages are produced.
pub trait XmlPropertyType: Sized {
    /// Try to read a value of type `Self` from an optional XML attribute value, or give
    /// a [String] explaining the error.
    ///
    ///  > The error `String` should be a full sentence in English (or sentences). Ideally, it
    ///  should also contain the current value of the XML attribute and an explanation why the
    ///  conversion failed for this specific value.
    ///
    /// Note that it is up to the implementation how it treats missing values. In some cases,
    /// there can be a suitable "default" value that is returned. In other cases,
    /// the appropriate result is an error.
    fn try_read(value: Option<&str>) -> Result<Option<Self>, String>;

    /// Convert the value of `Self` into a [String], or `None` when the value should be
    /// represented as a missing attribute.
    ///
    /// Note that in some cases, the "default" value of `Self` can correspond to multiple
    /// XML attribute values. For example, value `None` could be correctly represented both
    /// as a missing attribute and an empty string. In such case, prefer the less verbose
    /// option (missing attribute).
    fn write(&self) -> Option<String>;
}
