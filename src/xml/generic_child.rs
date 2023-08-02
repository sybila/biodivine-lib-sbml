use crate::xml::{XmlChild, XmlElement, XmlWrapper};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// [GenericChild] is an implementation of [XmlChild] that uses a child name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual children, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct GenericChild<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

impl<T: XmlWrapper> XmlChild<T> for GenericChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        let doc = self.element.read_doc();
        self.element
            .element()
            .find(doc.deref(), self.name.as_str())
            .is_some()
    }

    fn clear(&self) {
        let mut doc = self.element.write_doc();
        let parent = self.element.element();
        let Some(to_remove) = parent.find(doc.deref(), self.name.as_str()) else {
            return;
        };
        to_remove
            .detatch(doc.deref_mut())
            .expect("You can't detach the container element.");
    }

    fn get(&self) -> T {
        self.get_raw()
            .map(|it| it.into())
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn get_raw(&self) -> Option<XmlElement> {
        let doc = self.element.read_doc();
        let parent = self.element.element();
        parent
            .find(doc.deref(), self.name.as_str())
            .map(|it| XmlElement::new(self.element.document(), it))
    }

    fn set(&self, element: T) -> Option<T> {
        self.set_raw(element.into()).map(|it| it.into())
    }

    fn set_raw(&self, element: XmlElement) -> Option<XmlElement> {
        let mut doc = self.element.write_doc();
        let parent = self.element.element();

        // First, remove the existing child.
        let removed = if let Some(to_remove) = parent.find(doc.deref(), self.name.as_str()) {
            to_remove
                .detatch(doc.deref_mut())
                .expect("You can't detach the container element.");
            Some(XmlElement::new(self.element.document(), to_remove))
        } else {
            None
        };

        // Now, push the new child and check that the result is ok.
        let result = parent.push_child(doc.deref_mut(), element.element().as_node());
        match result {
            Err(xml_doc::Error::HasAParent) => {
                panic!("Cannot set child. The given element already has a parent.")
            }
            Err(xml_doc::Error::ContainerCannotMove) => {
                panic!("Cannot attach the container element to a parent.")
            }
            _ => (),
        };

        // Return the old child.
        removed
    }
}
