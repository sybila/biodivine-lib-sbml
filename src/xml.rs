use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use xml_doc::{Document, Element, Node};

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
pub type XmlDocument = Arc<RwLock<Document>>;

/// An [XmlElement] maintains a single thread-safe reference to an [Element] of a [Document].
///
/// Internally, this is achieved through a reference counted [RwLock] (see [XmlDocument]).
///
/// Note that a lot of the useful functionality of [XmlElement] is actually implemented
/// through the [XmlWrapper] trait (however, [XmlWrapper] can be also implemented by other types
/// derived from [XmlElement]).
#[derive(Clone, Debug)]
pub struct XmlElement {
    document: XmlDocument,
    element: Element,
}

impl XmlElement {
    /// Create a new [XmlElement] from an existing [XmlDocument] and [Element].
    pub fn new(document: XmlDocument, element: Element) -> XmlElement {
        XmlElement { document, element }
    }

    /// Create a new [XmlElement] from an existing [Document] and [Element].
    ///
    /// Note that this method takes the ownership of the whole XML [Document] and wraps it into
    /// a thread-safe [XmlDocument]. You can then create new [XmlElement] objects for this document
    /// using [XmlElement::derive].
    pub fn build(document: Document, element: Element) -> XmlElement {
        let document = Arc::new(RwLock::new(document));
        Self::new(document, element)
    }

    /// Create a new [XmlElement] which has the same underlying [XmlDocument] as *this*
    /// [XmlElement], but references a different [Element].
    pub fn derive(&self, element: Element) -> XmlElement {
        XmlElement {
            document: self.document.clone(),
            element,
        }
    }
}

impl XmlWrapper for XmlElement {
    fn as_xml(&self) -> &XmlElement {
        self
    }
}

/// This trait is implemented by all types that can behave as an [XmlElement] (including
/// [XmlElement] itself).
///
/// The trait also provides basic utility functions over the underlying [XmlElement] instance
/// so that one does not have to constantly call [XmlWrapper::as_xml] to perform any XMl operation.
pub trait XmlWrapper {
    /// Obtain a reference to the underlying [XmlElement].
    fn as_xml(&self) -> &XmlElement;

    /// Get the [Element] instance of the underlying [XmlElement].
    fn element(&self) -> Element {
        self.as_xml().element
    }

    /// Get a copy of the string value of the XML attribute of the given [name], or `None` if such
    /// attribute is not defined.
    fn get_attribute(&self, name: &str) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), name)
            .map(|it| it.to_string())
    }

    /// Set the [value] of the XML attribute of the given [name].
    fn set_attribute(&self, name: &str, value: String) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), name, value)
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

/// Implements a wrapper for XML elements that represent a typed list. That is, their child nodes
/// are all of one type and are supposed to be accessed using integer indices.
///
/// Note that we technically don't really *need* any `get_mut` or similar methods, since we
/// expect the elements to implement interior mutability through locks.
///
/// The actual type of the list items (generic parameter `Type`) has to implement
/// `From<XmlElement>` in order to ensure there is a way to convert the underlying list items
/// to the correct typed structures. Currently, this conversion is *not* allowed to fail (but it
/// can of course panic if the underlying element is in some sense invalid).
///
/// In the future, we need to think a bit more about how errors
/// should be handled in such cases. We'll probably need something like a `get` and `get_checked`
/// method, where `get` is allowed to panic, but `get_checked` will propagate the error making it
/// possible to detect and either fix or remove the invalid elements.
///
/// The other option is to never raise any errors when constructing objects from xml elements
/// and then have some kind of `validate` method that will go through the whole element tree
/// and check that the necessary conditions are satisfied.
///
/// TODO:
///     Overall, I'm marking "safety checks in generic XML lists" as a larger
///     TODO that needs to be discussed.
/// TODO:
///     Another thing that I realized just now: We might want to just require that
///     `From<XmlElement>` is actually implemented by every `XmlWrapper` type. However, this
///     will strongly depend on how we approach error handling in the future.
#[derive(Clone, Debug)]
pub struct XmlList<Type: From<XmlElement> + XmlWrapper> {
    element: XmlElement,
    // Rust actually does not allow generic type parameters which are not used inside the struct.
    // However, it is still a relatively common use case for generic collections like this one.
    // So they allow to specify that the type *should* be unused by using this special
    // `PhantomData` type that actually stores nothing.
    _marker: PhantomData<Type>,
}

/// Any [XmlList] type can be constructed from any [XmlElement]. The correctness of this
/// conversion will have to be checked at runtime.
impl<Type: From<XmlElement> + XmlWrapper> From<XmlElement> for XmlList<Type> {
    fn from(element: XmlElement) -> Self {
        XmlList::new(element)
    }
}

impl<Type: From<XmlElement> + XmlWrapper> XmlWrapper for XmlList<Type> {
    fn as_xml(&self) -> &XmlElement {
        &self.element
    }
}

// TODO:
//  There are a lot of `unwrap` calls in this code related to the general error handling
//  discussion concerning the `XmlList` type. These should be "resolved" once we have a better
//  idea how error handling will look like.
impl<Type: From<XmlElement> + XmlWrapper> XmlList<Type> {
    pub fn new(element: XmlElement) -> Self {
        XmlList {
            element,
            _marker: PhantomData,
        }
    }

    /// Get the element of this list at position `index`.
    ///
    /// Can panic if such element does not exist or it cannot be converted to `Type`.
    pub fn get(&self, index: usize) -> Type {
        let doc = self.read_doc();
        let result = self.element().children(doc.deref()).get(index).unwrap();
        // Here, we assume the XML node is a proper node (not text/etc.) and that it can be
        // converted to `Type`.
        let result = result.as_element().unwrap();
        let result = self.as_xml().derive(result);
        Type::from(result)
    }

    /// Insert a new element into the list. The element must not belong to an existing parent
    /// (i.e. it must be detached).
    pub fn insert(&self, index: usize, value: Type) {
        let mut doc = self.write_doc();
        let to_insert = Node::Element(value.element());
        self.element()
            .insert_child(doc.deref_mut(), index, to_insert)
            .unwrap();
    }

    /// Remove an element at the given position.
    pub fn remove(&self, index: usize) -> Type {
        let mut doc = self.write_doc();
        let removed = self.element().remove_child(doc.deref_mut(), index);
        // Here, we assume `removed` is a proper Xml element (i.e. not text or
        // other special element type). We also assume that it can be safely converted to `Type`
        // which may not be always true.
        let removed = removed.as_element().unwrap();
        let removed = self.as_xml().derive(removed);
        Type::from(removed)
    }

    /// Get number of elements contained in the list
    pub fn len(&self) -> usize {
        let doc = self.read_doc();
        self.element().child_elements(doc.deref()).len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Any [XmlProperty] object provides type-safe access to a single XML attribute
/// of an underlying tag.
///
/// [XmlProperty] objects maintain a reference to the underlying [XmlElement] with lifetime `'a`
/// and thus cannot be stored or passed around independently.
///
/// [XmlProperty] is also parametrized by a type `T` which is the underlying type of the property.
/// It is up to the implementation to perform a safe conversion between a string attribute and `T`.
///
/// In theory, an [XmlProperty] can be also backed by a combination of *multiple* XML attributes,
/// but this is not advised as the individual state each attribute is not clear in such case.
///
/// ## On missing attributes and value validity
///
/// Note that whether a missing value is considered valid is implementation specific and
/// depends on `T`. I.e. there can be types that always *require* some value, while there
/// can be types where a missing value represents some sort of default.
///
/// In general, it is recommended that when a missing value is considered valid but there
/// is no suitable `T::default()` value, we should have `T = Option<R>` (i.e. `T` is an
/// optional type). If there is a `T::default()` value, it is possible to return this value
/// when the attribute value is missing.
///
/// Similarly, when writing a property, if the property is optional (i.e. `T = Option<R>`), then
/// write functions are allowed to erase the attribute if `None` is being written, assuming there
/// is no other appropriate value that represents `None`. However, for other
/// instances, it is recommended to always write the full value, even if it equals the default
/// (i.e. missing) value. One can always explicitly remove the value by calling [Self::clear].
///
pub trait XmlProperty<'a, T> {

    /// Returns `true` if the underlying XML attribute has a known set value, even if such
    /// value is invalid.
    ///
    /// Note that this refers directly to value in the underlying document. When the attribute
    /// value is missing, this function must return `false`, even if a missing value
    /// is valid for type `T`.
    fn is_set(&self) -> bool {
        self.read_raw().is_some()
    }

    /// Returns `true` if the underlying XML attribute represents a valid value of type `T`.
    ///
    /// See the overall discussion in [XmlProperty] regarding how to treat validity of missing
    /// attribute values.
    fn is_valid(&self) -> bool {
        self.read_checked().is_some()
    }

    /// Read the value of this [XmlProperty].
    ///
    /// # Panics
    ///
    /// The function should panic if the underlying attribute value is invalid for type `T`.
    fn read(&self) -> T {
        self.read_checked().unwrap()
    }

    /// Read the value of this [XmlProperty], or `None` if the underlying value is invalid.
    ///
    /// See the overall discussion in [XmlProperty] regarding how to treat validity of missing
    /// attribute values.
    fn read_checked(&self) -> Option<T>;

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

// TODO: We will need a macro that implements XmlProperty for us. For that we need derive
// macros with arguments: https://stackoverflow.com/questions/56188700/how-do-i-make-my-custom-derive-macro-accept-trait-generic-parameters
