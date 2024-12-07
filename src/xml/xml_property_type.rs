/// [XmlPropertyType] is implemented by types that can be converted to/from an XML attribute.
///
/// Ideally, this conversion should only depend on the value of the XML attribute. I.e. it should
/// not check consistency with other parts of the document. There should be other mechanisms that
/// implement such checks where necessary, but this should not be included in the "read
/// a value" portion of the process.
///
///  > Technically, this is almost the same as implementing the `TryFrom` and `Into` traits, but
/// > should give us a bit more flexibility and clarity regarding what conversions happen where
/// > and what error messages are produced. In particular, we can have custom error messages and
/// > conversion rules just for XML.
///
/// ### Missing and optional values
///
/// The raw value of the XML attribute obtained from the document can be always missing
/// (`Option<&str>`). The conversion process can also always produce a missing value
/// (`Option<Self>` when reading, `Option<String>` when writing). However, the implementations
/// are not required to use these optional values if the conversion is infallible:
///
/// **Reading:** If there is a suitable default value available, the conversion can return
/// this default value instead of `None` (in which case, `None` is never returned by
/// `try_read`). For simplicity, this is not reflected in the function signature (there is no
/// "property type with default value" trait). Instead, the implementations of `XmlPropertyType`
/// should clearly explain their treatment of default values in their documentation.
///
/// **Writing:** Similarly, when writing a value into a [String], the implementation can choose
/// to serialize any default value as `None` if appropriate (for example, if appropriate, an empty
/// string can be automatically converted to `None`, resulting in an erased attribute). Ideally,
/// prefer the less verbose output during conversion (i.e. missing attribute, not empty string).
///
/// Due to the rules outlined above, it is rarely required to implement [XmlPropertyType] for
/// optional types. Instead, the "optional-ness" is handled through a higher level abstraction,
/// such as the XML property and child traits. In particular, in this setting, a missing
/// attribute value is never read as an error.
pub trait XmlPropertyType: Sized {
    /// Try to read a value of type `Self` from an optional XML attribute value, or give
    /// a [String] explaining the error.
    ///
    ///  > The error `String` should be a full sentence in English (or sentences). Ideally, it
    /// > should also contain the current value of the XML attribute and an explanation why the
    /// > conversion failed for this specific value. The message can use Markdown syntax for
    /// > formatting (e.g. code/highlights, even paragraphs/headers if really necessary).
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String>;

    /// Convert the value of `Self` into a [String], or `None` when the value should be
    /// represented as a missing attribute.
    fn set(&self) -> Option<String>;
}
