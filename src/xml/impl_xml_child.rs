use crate::xml::{XmlChild, XmlChildDefault, XmlElement, XmlList, XmlWrapper};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use xml_doc::Element;

use super::xml_child::XmlChildOptional;

/// [DynamicChild] is an implementation of [XmlChild] that uses a child name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual children, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct DynamicChild<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

pub struct DynamicChildOptional<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

/// [Child] is an implementation of [XmlChild] that uses a tag name that is known
/// at compile time. As such, it is faster than [DynamicChild], but less flexible.
pub struct Child<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: &'static str,
    _marker: PhantomData<T>,
}

pub struct ChildOptional<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: XmlWrapper> DynamicChild<'_, T> {
    /// Create a new instance of a [DynamicChild] for the given `element` and `name`.
    pub fn new<'a>(element: &'a XmlElement, name: &str) -> DynamicChild<'a, T> {
        DynamicChild {
            element,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }

    /// Read the name of this [DynamicChild].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: XmlWrapper> DynamicChildOptional<'_, T> {
    pub fn new<'a>(element: &'a XmlElement, name: &str) -> DynamicChildOptional<'a, T> {
        DynamicChildOptional {
            element,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: XmlWrapper> Child<'_, T> {
    pub fn new<'a>(element: &'a XmlElement, name: &'static str) -> Child<'a, T> {
        Child {
            element,
            name,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T: XmlWrapper> ChildOptional<'_, T> {
    pub fn new<'a>(element: &'a XmlElement, name: &'static str) -> ChildOptional<'a, T> {
        ChildOptional {
            element,
            name,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T: XmlWrapper> XmlChild<T> for DynamicChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        is_set(self.element, self.name())
    }

    fn clear(&self) {
        clear(self.element, self.name());
    }

    fn get(&self) -> T {
        get(self.element, self.name())
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn get_raw(&self) -> XmlElement {
        get_raw(self.element, self.name())
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn set(&self, element: T) -> T {
        set(self.element, self.name(), element)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn set_raw(&self, element: XmlElement) -> XmlElement {
        set_raw(self.element, self.name(), element)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }
}

impl<T: XmlWrapper> XmlChild<T> for Child<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        is_set(self.element, self.name)
    }

    fn clear(&self) {
        clear(self.element, self.name)
    }

    fn get(&self) -> T {
        get(self.element, self.name)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn get_raw(&self) -> XmlElement {
        get_raw(self.element, self.name)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn set(&self, element: T) -> T {
        set(self.element, self.name, element)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }

    fn set_raw(&self, element: XmlElement) -> XmlElement {
        set_raw(self.element, self.name, element)
            .unwrap_or_else(|| panic!("Child element `{}` is missing.", self.name))
    }
}

impl<T: XmlWrapper> XmlChildOptional<T> for DynamicChildOptional<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        is_set(self.element, self.name())
    }

    fn clear(&self) {
        clear(self.element, self.name())
    }

    fn get(&self) -> Option<T> {
        get(self.element, self.name())
    }

    fn get_raw(&self) -> Option<XmlElement> {
        get_raw(self.element, self.name())
    }

    fn set(&self, element: T) -> Option<T> {
        set(self.element, self.name(), element)
    }

    fn set_raw(&self, element: XmlElement) -> Option<XmlElement> {
        set_raw(self.element, self.name(), element)
    }
}

impl<T: XmlWrapper> XmlChildOptional<T> for ChildOptional<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        is_set(self.element, self.name)
    }

    fn clear(&self) {
        clear(self.element, self.name)
    }

    fn get(&self) -> Option<T> {
        get(self.element, self.name)
    }

    fn get_raw(&self) -> Option<XmlElement> {
        get_raw(self.element, self.name)
    }

    fn set(&self, element: T) -> Option<T> {
        set(self.element, self.name, element)
    }

    fn set_raw(&self, element: XmlElement) -> Option<XmlElement> {
        set_raw(self.element, self.name, element)
    }
}

impl<Inner: XmlWrapper> XmlChildDefault<XmlList<Inner>>
    for DynamicChildOptional<'_, XmlList<Inner>>
{
    fn ensure(&self) {
        ensure(self.element, self.name())
    }
}

impl<Inner: XmlWrapper> XmlChildDefault<XmlList<Inner>> for ChildOptional<'_, XmlList<Inner>> {
    fn ensure(&self) {
        ensure(self.element, self.name())
    }
}

/*
   The following functions implement [XmlChild] in both the [GenericChild] and
   all macro implementations. They are only visible to the crate code (`pub(crate)`),
   i.e. they are private within this library.
*/

fn is_set(element: &XmlElement, name: &str) -> bool {
    let doc = element.read_doc();
    element.element().find(doc.deref(), name).is_some()
}

fn clear(element: &XmlElement, name: &str) {
    let mut doc = element.write_doc();
    let parent = element.element();
    let Some(to_remove) = parent.find(doc.deref(), name) else {
        return;
    };
    to_remove
        .detatch(doc.deref_mut())
        .expect("You can't detach the container element.");
}

fn get<T: XmlWrapper>(element: &XmlElement, name: &str) -> Option<T> {
    get_raw(element, name).map(|it| it.into())
    // .unwrap_or_else(|| panic!("Child element `{}` is missing.", name))
}

fn get_raw(element: &XmlElement, name: &str) -> Option<XmlElement> {
    let doc = element.read_doc();
    let parent = element.element();
    parent
        .find(doc.deref(), name)
        .map(|it| XmlElement::new(element.document(), it))
}

fn set<T: XmlWrapper>(element: &XmlElement, name: &str, value: T) -> Option<T> {
    set_raw(element, name, value.into()).map(|it| it.into())
}

fn set_raw(element: &XmlElement, name: &str, value: XmlElement) -> Option<XmlElement> {
    let mut doc = element.write_doc();
    let parent = element.element();

    // First, remove the existing child.
    let removed = if let Some(to_remove) = parent.find(doc.deref(), name) {
        to_remove
            .detatch(doc.deref_mut())
            .expect("You can't detach the container element.");
        Some(XmlElement::new(element.document(), to_remove))
    } else {
        None
    };

    // Now, push the new child and check that the result is ok.
    let value = value.element.as_node();
    let result = parent.push_child(doc.deref_mut(), value);
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

fn ensure(parent: &XmlElement, name: &str) {
    if !is_set(parent, name) {
        let mut doc = parent.write_doc();
        let element = Element::new(doc.deref_mut(), name);
        set_raw(parent, name, XmlElement::new(parent.document(), element));
    }
}
