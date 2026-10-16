use crate::xml::{
    OptionalDynamicChild, OptionalDynamicProperty, RequiredDynamicChild, RequiredDynamicProperty,
    XmlElement, XmlPropertyType,
};
use biodivine_lib_xml_dom::xml_spec::Text;
use biodivine_lib_xml_dom::{Document, QualifiedName};
use std::collections::BTreeMap;

/// [XmlWrapper] is implemented by every type that provides a typed view of an [XmlElement]
/// (including [XmlElement] itself).
///
/// The wrapper is intentionally shallow: the only genuinely required method is
/// [XmlWrapper::xml_element], plus the two `From` conversions between the wrapper type and
/// [XmlElement]. The remaining methods are one-line conveniences that forward to the
/// corresponding inherent method of [XmlElement], so that SBML objects can be used directly
/// without reaching for `xml_element()` at every call site.
///
/// The type safety offered by a wrapper is checked when the properties or children of the
/// element are read or written; creating a wrapper does not by itself verify that the element
/// has the expected structure. Converting an arbitrary [XmlElement] into a wrapper `T` is a
/// plain `T::from(element)`; whether the result is a *valid* `T` is decided by the validation
/// layer, not by the conversion.
pub trait XmlWrapper: From<XmlElement> + Into<XmlElement> {
    /// Obtain a reference to the underlying [XmlElement].
    fn xml_element(&self) -> &XmlElement;

    /// The document to which the underlying element belongs.
    fn document(&self) -> Document {
        self.xml_element().document()
    }

    /// The expanded name of the underlying element.
    fn qualified_name(&self) -> QualifiedName {
        self.xml_element().qualified_name()
    }

    /// The local name of the underlying element.
    fn tag_name(&self) -> String {
        self.xml_element().tag_name()
    }

    /// The fully qualified name of the underlying element, including a namespace prefix.
    fn full_name(&self) -> String {
        self.xml_element().full_name()
    }

    /// The namespace URL of the underlying element (empty string if it has no namespace).
    fn namespace_url(&self) -> String {
        self.xml_element().namespace_url()
    }

    /// The attributes of the underlying element, keyed by expanded name.
    fn attributes(&self) -> BTreeMap<QualifiedName, Text> {
        self.xml_element().attributes()
    }

    /// True if the underlying element has an attribute with the given local name.
    fn has_attribute(&self, name: &str) -> bool {
        self.xml_element().has_attribute(name)
    }

    /// The value of the attribute with the given local name, if it is present.
    fn get_attribute(&self, name: &str) -> Option<String> {
        self.xml_element().get_attribute(name)
    }

    /// The concatenated text content of the underlying element and its descendants.
    fn text_content(&self) -> String {
        self.xml_element().text_content()
    }

    /// The parent element of the underlying element, if any.
    fn parent(&self) -> Option<XmlElement> {
        self.xml_element().parent()
    }

    /// The immediate child elements of the underlying element.
    fn child_elements(&self) -> Vec<XmlElement> {
        self.xml_element().child_elements()
    }

    /// A filtered version of [XmlWrapper::child_elements].
    fn child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        predicate: P,
    ) -> Vec<XmlElement> {
        self.xml_element().child_elements_filtered(predicate)
    }

    /// The `index`-th child element of the underlying element.
    fn get_child_at(&self, index: usize) -> Option<XmlElement> {
        self.xml_element().get_child_at(index)
    }

    /// All descendant elements of the underlying element, in document order.
    fn recursive_child_elements(&self) -> Vec<XmlElement> {
        self.xml_element().recursive_child_elements()
    }

    /// A filtered version of [XmlWrapper::recursive_child_elements].
    fn recursive_child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        predicate: P,
    ) -> Vec<XmlElement> {
        self.xml_element()
            .recursive_child_elements_filtered(predicate)
    }

    /// A [RequiredDynamicProperty] for the attribute with the given name (no namespace).
    fn required_property<T: XmlPropertyType>(&self, name: &str) -> RequiredDynamicProperty<T> {
        RequiredDynamicProperty::new(self.xml_element(), name)
    }

    /// An [OptionalDynamicProperty] for the attribute with the given name (no namespace).
    fn optional_property<T: XmlPropertyType>(&self, name: &str) -> OptionalDynamicProperty<T> {
        OptionalDynamicProperty::new(self.xml_element(), name)
    }

    /// An [OptionalDynamicChild] for the child element with the given name (no namespace).
    fn optional_child<T: XmlWrapper>(&self, name: &str) -> OptionalDynamicChild<T> {
        OptionalDynamicChild::new(self.xml_element(), name)
    }

    /// A [RequiredDynamicChild] for the child element with the given name (no namespace).
    fn required_child<T: XmlWrapper>(&self, name: &str) -> RequiredDynamicChild<T> {
        RequiredDynamicChild::new(self.xml_element(), name)
    }
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
