use crate::core::SBase;
use crate::xml::{XmlElement, XmlWrapper};
use std::marker::PhantomData;

/// [XmlList] is a generic implementation of [XmlWrapper] that represents a typed list of
/// elements. That is, each child node of the wrapped tag represents one instance of
/// a particular [XmlWrapper] type.
///
/// The actual type of the list items (generic parameter `Type`) has to implement
/// [XmlWrapper] so that each child element can be wrapped into the correct typed structure.
/// This conversion is infallible: an element is wrapped as `Type` regardless of whether it
/// actually is a valid `Type`, and validity is decided by the validation layer.
///
/// The list tag must not contain any other content other than the list items.
#[derive(Clone, Debug)]
pub struct XmlList<Type: XmlWrapper> {
    element: XmlElement,
    // Rust does not allow a generic type parameter which is not used inside the struct, and a
    // generic collection needs one. `PhantomData` is the standard way of saying "this type
    // parameter is intentionally unused".
    _marker: PhantomData<Type>,
}

impl<Type: XmlWrapper> From<XmlList<Type>> for XmlElement {
    fn from(element: XmlList<Type>) -> Self {
        element.element
    }
}

impl<Type: XmlWrapper> From<XmlElement> for XmlList<Type> {
    fn from(element: XmlElement) -> Self {
        XmlList {
            element,
            _marker: PhantomData,
        }
    }
}

impl<Type: XmlWrapper> XmlWrapper for XmlList<Type> {
    fn xml_element(&self) -> &XmlElement {
        &self.element
    }
}

impl<Type: XmlWrapper> XmlList<Type> {
    /// Map an "outside" index referencing a child element to an "inside" index referencing
    /// a proper XML node (i.e. accounting for text and comments).
    ///
    /// Returns `None` if the index does not exist.
    fn remap_index(&self, mut outside_index: usize) -> Option<usize> {
        for (inside_index, child) in self.element.element().children().iter().enumerate() {
            if child.as_element().is_some() {
                if outside_index == 0 {
                    return Some(inside_index);
                }
                outside_index -= 1;
            }
        }
        None
    }

    /// Get the element of this list at the position specified by `index`.
    ///
    /// # Panics
    ///
    /// Panics if the specified item does not exist.
    pub fn get(&self, index: usize) -> Type {
        self.get_checked(index)
            .unwrap_or_else(|| panic!("No XML element at position {index}."))
    }

    /// Get the element of this list at the position specified by `index`, or `None`
    /// if such position does not exist.
    pub fn get_checked(&self, index: usize) -> Option<Type> {
        self.element.get_child_at(index).map(Type::from)
    }

    /// Insert a new element into the list at the given position.
    ///
    /// # Panics
    ///
    /// Panics if `value` cannot be attached to the list tag (it is the document root, or it
    /// belongs to another document).
    pub fn insert(&self, index: usize, value: Type) {
        let index = self
            .remap_index(index)
            .unwrap_or_else(|| self.element.element().children().len());
        let value: XmlElement = value.into();
        self.element
            .element()
            .insert_child(index, value.into_element().node());
    }

    /// Remove an element at the given position and return the removed value.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn remove(&self, index: usize) -> Type {
        let Some(element) = self.element.get_child_at(index) else {
            panic!("Item at position {index} does not exist.");
        };
        element.element().detach();
        Type::from(element)
    }

    /// Insert a new element into the list at the last position similarly as in stack.
    ///
    /// # Panics
    ///
    /// Panics if `value` cannot be attached to the list tag.
    pub fn push(&self, value: Type) {
        self.insert(self.len(), value)
    }

    /// Remove an element from the last position similarly as in stack, and return it.
    ///
    /// # Panics
    ///
    /// Panics if the list is already empty.
    pub fn pop(&self) -> Type {
        self.remove(self.len() - 1)
    }

    /// Get an element from the last position similarly as in stack, but without removing.
    ///
    /// # Panics
    ///
    /// Panics if the list is already empty.
    pub fn top(&self) -> Type {
        self.get(self.len() - 1)
    }

    /// Get the number of elements contained in the list.
    pub fn len(&self) -> usize {
        self.element.child_elements().len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A vector of all the elements contained in the list, in order.
    pub fn as_vec(&self) -> Vec<Type> {
        self.element
            .child_elements()
            .into_iter()
            .map(Type::from)
            .collect()
    }

    /// An iterator over the elements of the list.
    pub fn iter(&self) -> XmlListIterator<'_, Type> {
        XmlListIterator {
            list: self,
            index: 0,
        }
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

/// A helper structure which allows us to iterate over the elements of a [XmlList].
pub struct XmlListIterator<'a, T: XmlWrapper> {
    list: &'a XmlList<T>,
    index: usize,
}

impl<T: XmlWrapper> Iterator for XmlListIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.len() {
            None
        } else {
            let item = self.list.get(self.index);
            self.index += 1;
            Some(item)
        }
    }
}
