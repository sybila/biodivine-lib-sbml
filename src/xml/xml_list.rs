use crate::core::SBase;
use crate::xml::{XmlElement, XmlWrapper};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// [XmlList] is a generic implementation of [XmlWrapper] that represents a typed list of
/// elements. That is, each child node of the wrapped tag represents one instance of
/// a particular [XmlWrapper] type.
///
/// The actual type of the list items (generic parameter `Type`) has to implement
/// `XmlWrapper` (which required `From<XmlElement>`) in order to ensure there is a way to convert
/// the underlying list tags to the correct typed structures. Currently, this conversion is *not*
/// allowed to fail.
///
/// The list tag must not contain any other content other than the list items.
///
///
/// TODO:
///     XmlList certainly needs to implement validation. Possibly incomplete list of conditions
///     that we should check:
///         - The only child nodes are `Element` nodes (no text or similar).
///         - Run validation on all child elements if they implement the appropriate traits.
#[derive(Clone, Debug)]
pub struct XmlList<Type: XmlWrapper> {
    element: XmlElement,
    // Rust actually does not allow generic type parameters which are not used inside the struct.
    // However, it is still a relatively common use case for generic collections like this one.
    // So they allow to specify that the type *should* be unused by using this special
    // `PhantomData` type that actually stores nothing.
    _marker: PhantomData<Type>,
}

impl<Type: XmlWrapper> From<XmlList<Type>> for XmlElement {
    fn from(element: XmlList<Type>) -> Self {
        element.element
    }
}

impl<Type: XmlWrapper> XmlWrapper for XmlList<Type> {
    fn xml_element(&self) -> &XmlElement {
        &self.element
    }

    unsafe fn unchecked_cast<T: XmlWrapper>(element: T) -> Self {
        XmlList {
            element: element.xml_element().clone(),
            _marker: PhantomData,
        }
    }
}

impl<Type: XmlWrapper> XmlList<Type> {
    /// Get the element of this list at the position specified by `index`.
    ///
    /// # Panics
    ///
    /// Panics if the specified item does not exist, or if the XML node at the given position
    /// is not an element (for example text).
    pub fn get(&self, index: usize) -> Type {
        self.get_checked(index)
            .unwrap_or_else(|| panic!("No XML element at position {index}."))
    }

    /// Get the element of this list at the position specified by `index`, or `None`
    /// if such position does not exist.
    ///
    /// # Panics
    ///
    /// The method panics if the list tag contains additional content that is not an
    /// XML tag (e.g. text).
    pub fn get_checked(&self, index: usize) -> Option<Type> {
        let doc = self.read_doc();
        self.raw_element()
            .children(doc.deref())
            .get(index)
            .map(|it| {
                let element = it.as_element().unwrap_or_else(|| {
                    panic!("Item at position {index} is not an XML element.");
                });
                unsafe {
                    // TODO: This really is not safe at the moment.
                    Type::unchecked_cast(XmlElement::new_raw(self.document(), element))
                }
            })
    }

    /// Insert a new element into the list. The element must not belong to an existing parent
    /// (i.e. it must be detached).
    ///
    /// # Panics
    ///
    /// Panics if `index > len`, or when `value` cannot be attached to the list tag
    /// (it already has a parent, or is itself the root container tag).
    pub fn insert(&self, index: usize, value: Type) {
        value.try_attach_at(self, Some(index)).unwrap();
    }

    /// Remove an element at the given position and return the removed value.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`, or if the XML node at the given position
    /// is not an element (for example text).
    pub fn remove(&self, index: usize) -> Type {
        let mut doc = self.write_doc();
        let removed = self.raw_element().remove_child(doc.deref_mut(), index);
        // Here, we assume `removed` is a proper Xml element (i.e. not text or
        // other special element type). We also assume that it can be safely converted to `Type`
        // which may not be always true.
        let removed = removed.as_element().unwrap_or_else(|| {
            panic!("Item at position {index} is not an XML element.");
        });
        unsafe {
            // TODO: This really is not safe at the moment.
            Type::unchecked_cast(XmlElement::new_raw(self.document(), removed))
        }
    }

    /// Insert a new element into the list at the last position similarly as in stack.
    ///
    /// # Panics
    /// Panics if `value` cannot be attached to the list tag (it already has a parent,
    /// or is itself the root container tag).
    pub fn push(&self, value: Type) {
        self.insert(self.len(), value)
    }

    /// Remove an element from the last position similarly as in stack, and return it.
    ///
    /// # Panics
    ///
    /// Panics if the list is already empty or if the XML node at the given position
    /// is not an element (for example text).
    pub fn pop(&self) -> Type {
        self.remove(self.len() - 1)
    }

    /// Get an element from the last position similarly as in stack, but without removing.
    ///
    /// # Panics
    ///
    /// Panics if the list is already empty or if the XML node at the given position
    /// is not an element (for example text).
    pub fn top(&self) -> Type {
        self.get(self.len() - 1)
    }

    /// Get the number of elements contained in the list.
    pub fn len(&self) -> usize {
        let doc = self.read_doc();
        self.raw_element().child_elements(doc.deref()).len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// TODO:
//   This is fine for now, but I would very much like to remove this in the future.
//   The problem is that now `XmlList` can be used *only* in places where it implements `SBase`.
//   So any list of objects that are not `SBase` should not be represented as `XmlList`.
//   A possible solution would be to implement `XmlList` as a trait, and then have a `SbmlList`
//   struct that implements it together with `SBase`, and possibly other implementations that
//   do not use `SBase`.
impl<T: XmlWrapper> SBase for XmlList<T> {}
