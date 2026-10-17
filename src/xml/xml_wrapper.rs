use crate::xml::XmlElement;
use biodivine_lib_xml_dom::Document;

/// [XmlWrapper] is implemented by every type that provides a typed view of an [XmlElement]
/// (including [XmlElement] itself).
///
/// The wrapper is intentionally shallow: the only required method is [XmlWrapper::xml_element],
/// plus the two `From` conversions between the wrapper type and [XmlElement]. The trait is the
/// bound that makes the object model generic over SBML object types: typed children and lists, the
/// subtype/supertype casts, and the package-declaration helpers all work with any [XmlWrapper],
/// while the element API itself lives in exactly one place, on [XmlElement].
///
/// The type safety offered by a wrapper is checked when the properties or children of the element
/// are read or written; creating a wrapper does not by itself verify that the element has the
/// expected structure. Converting an arbitrary [XmlElement] into a wrapper `T` is a plain
/// `T::from(element)`; whether the result is a *valid* `T` is decided by the validation layer, not
/// by the conversion.
pub trait XmlWrapper: From<XmlElement> + Into<XmlElement> {
    /// Obtain a reference to the underlying [XmlElement].
    fn xml_element(&self) -> &XmlElement;
}

/// [XmlDefault] extends [XmlWrapper] with the ability to build a "default" value of `Self`
/// (an element in a detached state) within a given [Document].
///
/// The result should represent a reasonable starting point for the type, but it does not have
/// to be fully valid with respect to the global state of the document (for example, a newly
/// assigned identifier may not be unique).
pub trait XmlDefault: XmlWrapper {
    /// Construct a "default" value of this type in the provided [Document].
    fn default(document: Document) -> Self;
}
