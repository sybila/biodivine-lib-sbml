use crate::xml::{GenericProperty, XmlDocument, XmlElement, XmlPropertyType};
use std::ops::Deref;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use xml_doc::{Document, Element};

/// [XmlWrapper] is implemented by all types that can behave as an [XmlElement]
/// (including [XmlElement] itself). In other words, all types that provide some "typed"
/// access to an underlying XML element.
///
/// The trait also provides basic utility functions over the underlying [XmlElement] instance
/// so that one does not have to constantly call [XmlWrapper::as_xml] to perform any XMl operation.
pub trait XmlWrapper: From<XmlElement> + Into<XmlElement> {
    /// Obtain a reference to the underlying [XmlElement].
    fn as_xml(&self) -> &XmlElement;

    /// Get the [Element] instance of the underlying [XmlElement].
    fn element(&self) -> Element {
        self.as_xml().element
    }

    /// Obtain a (counted) reference to the underlying [XmlDocument].
    fn document(&self) -> XmlDocument {
        self.as_xml().document.clone()
    }

    /// Get a reference to a specific [XmlProperty] of this XML element.
    ///
    /// Note that individual [XmlWrapper] implementations should provide type safe access
    /// to their known/required properties through specialised [XmlProperty] implementations
    /// instead of relying on [GenericProperty].
    fn property<T: XmlPropertyType>(&self, name: &str) -> GenericProperty<T> {
        GenericProperty::new(self.as_xml(), name)
    }

    /// Obtain a read-only reference to the underlying [Document].
    fn read_doc(&self) -> RwLockReadGuard<Document> {
        // Error handling note: In general, lock access will fail only when some other part
        // of the program performed an incorrect unsafe action (e.g. double release of the
        // same lock guard). As such, it is generally ok to panic here, because at that point
        // the whole document might be corrupted and we have no way to recover.
        self.as_xml()
            .document
            .read()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Obtain a writeable reference to the underlying [Document].
    fn write_doc(&self) -> RwLockWriteGuard<Document> {
        // See [Self::read_doc] for error handling notes.
        self.as_xml()
            .document
            .write()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    fn child_element(&self, name: &str) -> Element {
        self.element().find(self.read_doc().deref(), name).unwrap()
    }
}

/// [XmlDefault] extends the functionality of [XmlWrapper] by providing a method that can build
/// a "default" value of `Self` in the given [XmlDocument].
///
/// The resulting element should be in a "detached" state, meaning it has no parent.
///
/// Ideally, the result should represent a valid value of type `Self`. However, it does not have
/// to take into account the global state of the document (e.g. if there is an ID, it may not be
/// unique).
pub trait XmlDefault: XmlWrapper {
    /// Construct a "default" value of this type in the provided [XmlDocument].
    fn default(document: XmlDocument) -> Self;
}
