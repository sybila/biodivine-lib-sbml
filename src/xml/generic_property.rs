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
    /// Create a new instance of a [GenericProperty] for a given `element` and `name`.
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
        // As opposed to `self.read_raw().is_some()`, this does not need to copy.
        let doc = self.element.read_doc();
        self.element
            .element()
            .attribute(doc.deref(), self.name.as_str())
            .is_some()
    }

    fn is_valid(&self) -> bool {
        self.read_checked().is_ok()
    }

    fn read(&self) -> T {
        match self.read_checked() {
            Ok(result) => result,
            Err(message) => {
                panic!("Cannot read property `{}`: {}", self.name, message)
            }
        }
    }

    fn read_checked(&self) -> Result<T, String> {
        let doc = self.element.read_doc();
        let value = self
            .element
            .element()
            .attribute(doc.deref(), self.name.as_str());
        XmlPropertyType::try_read(value)
    }

    fn read_raw(&self) -> Option<String> {
        let doc = self.element.read_doc();
        self.element
            .element()
            .attribute(doc.deref(), self.name.as_str())
            .map(|it| it.to_string())
    }

    fn clear(&self) {
        let mut doc = self.element.write_doc();
        self.element
            .element()
            .mut_attributes(doc.deref_mut())
            .remove(&self.name);
    }

    fn write(&self, value: &T) {
        if let Some(value) = XmlPropertyType::write(value) {
            self.write_raw(value);
        } else {
            self.clear();
        }
    }

    fn write_raw(&self, value: String) {
        let mut doc = self.element.write_doc();
        self.element
            .element()
            .set_attribute(doc.deref_mut(), self.name.as_str(), value);
    }
}
