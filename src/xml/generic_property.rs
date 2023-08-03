use crate::xml::{XmlElement, XmlProperty, XmlPropertyType, XmlWrapper};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// [GenericProperty] is an implementation of [XmlProperty] that uses an attribute name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual properties, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct GenericProperty<'a, T: XmlPropertyType> {
    element: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

impl<'a, T: XmlPropertyType> GenericProperty<'a, T> {
    /// Create a new instance of a [GenericProperty] for the given `element` and `name`.
    pub fn new(element: &'a XmlElement, name: &str) -> GenericProperty<'a, T> {
        GenericProperty {
            element,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }

    /// Read the name of this [GenericProperty].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for GenericProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.element
    }

    fn is_set(&self) -> bool {
        is_set(self.element, self.name())
    }

    fn read(&self) -> T {
        read(self.element, self.name())
    }

    fn read_checked(&self) -> Result<T, String> {
        read_checked(self.element, self.name())
    }

    fn read_raw(&self) -> Option<String> {
        read_raw(self.element, self.name())
    }

    fn clear(&self) {
        clear(self.element, self.name());
    }

    fn write(&self, value: &T) {
        write(self.element, self.name(), value);
    }

    fn write_raw(&self, value: String) {
        write_raw(self.element, self.name(), value);
    }
}

/*
   The following functions implement [XmlProperty] in both the [GenericProperty] and
   all macro implementations. They are only visible to the crate code (`pub(crate)`),
   i.e. they are private within this library. They are inlined just to make extra sure
   the string names are not re-allocated when not necessary.
*/

pub(crate) fn is_set(element: &XmlElement, name: &str) -> bool {
    // As opposed to `self.read_raw().is_some()`, this does not need to copy.
    let doc = element.read_doc();
    element.element().attribute(doc.deref(), name).is_some()
}

pub(crate) fn read<T: XmlPropertyType>(element: &XmlElement, name: &str) -> T {
    match read_checked(element, name) {
        Ok(result) => result,
        Err(message) => {
            panic!("Cannot read property `{}`: {}", name, message)
        }
    }
}

pub(crate) fn read_checked<T: XmlPropertyType>(
    element: &XmlElement,
    name: &str,
) -> Result<T, String> {
    let doc = element.read_doc();
    let value = element.element().attribute(doc.deref(), name);
    XmlPropertyType::try_read(value)
}

pub(crate) fn read_raw(element: &XmlElement, name: &str) -> Option<String> {
    let doc = element.read_doc();
    element
        .element()
        .attribute(doc.deref(), name)
        .map(|it| it.to_string())
}

pub(crate) fn clear(element: &XmlElement, name: &str) {
    let mut doc = element.write_doc();
    element
        .element()
        .mut_attributes(doc.deref_mut())
        .remove(name);
}

pub(crate) fn write<T: XmlPropertyType>(element: &XmlElement, name: &str, value: &T) {
    if let Some(value) = XmlPropertyType::write(value) {
        write_raw(element, name, value);
    } else {
        clear(element, name);
    }
}

pub(crate) fn write_raw(element: &XmlElement, name: &str, value: String) {
    let mut doc = element.write_doc();
    element
        .element()
        .set_attribute(doc.deref_mut(), name, value);
}
