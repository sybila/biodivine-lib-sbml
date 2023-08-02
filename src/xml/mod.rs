use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use xml_doc::{Document, Node};

/// Implementation of the [XmlElement] struct.
mod xml_element;

/// Declaration of the [XmlWrapper] and [XmlDefault] traits.
mod xml_wrapper;

/// Declaration of the [XmlPropertyType] trait.
mod xml_property_type;

/// Declaration of the [XmlProperty] trait.
mod xml_property;

/// Declaration of the [XmlChild] and [XmlChildDefault] traits.
mod xml_child;

/// Implementation of the [GenericProperty] struct.
mod generic_property;

/// Implementation of the [GenericChild] struct.
mod generic_child;

/// Some primitive [XmlPropertyType] implementations, as declared in SBML
/// specification Section 3.1.
mod impl_property_type;

pub use crate::xml::xml_child::XmlChild;
pub use crate::xml::xml_child::XmlChildDefault;
pub use crate::xml::xml_element::XmlElement;
pub use crate::xml::xml_property::XmlProperty;
pub use crate::xml::xml_property_type::XmlPropertyType;
pub use crate::xml::xml_wrapper::XmlDefault;
pub use crate::xml::xml_wrapper::XmlWrapper;

pub use crate::xml::generic_child::GenericChild;
pub use crate::xml::generic_property::GenericProperty;

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
pub type XmlDocument = Arc<RwLock<Document>>;

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

impl<Type: From<XmlElement> + XmlWrapper> From<XmlList<Type>> for XmlElement {
    fn from(element: XmlList<Type>) -> Self {
        element.element
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
        Type::from(XmlElement::new(self.document(), result))
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
        Type::from(XmlElement::new(self.document(), removed))
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
