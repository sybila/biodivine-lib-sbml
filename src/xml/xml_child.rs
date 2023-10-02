use crate::xml::{XmlDefault, XmlElement, XmlList, XmlWrapper};
use std::ops::{Deref, DerefMut};

/// [XmlChild] implements a reference to a singleton child element `T`. That is, an element
/// which is unique in its parent and represents a larger structure of type `T`.
///
/// There are two variants of [XmlChild]: [OptionalXmlChild] and [RequiredXmlChild]. These
/// implement the two typical types of child elements.
///
/// *Warning:* At the moment, [XmlChild] implementations do not check that the child element
/// is truly a singleton. Unexpected behaviour can occur if this is not the case. Ideally,
/// this condition should be checked by additional document-wide validation steps.
pub trait XmlChild<T: XmlWrapper> {
    /// Returns a reference to the underlying parent [XmlElement].
    fn parent(&self) -> &XmlElement;

    /// Returns the name of the corresponding child tag.
    fn name(&self) -> &str;

    /// Returns the URL of the namespace where this child belongs. The url can be empty,
    /// in which case it translates to the "default namespace".
    fn namespace_url(&self) -> &str;

    /// Returns `true` if the referenced child element exists
    /// (even if it is otherwise invalid).
    fn is_set(&self) -> bool {
        let element = self.parent();
        let doc = element.read_doc();
        let (name, namespace) = (self.name(), self.namespace_url());
        element
            .element()
            .find_quantified(doc.deref(), name, namespace)
            .is_some()
    }

    /// Completely remove the referenced child element and return it (if it is present).
    ///
    /// # Document validity
    ///
    /// If this particular child is a required part of the document, this may cause the
    /// document to become invalid.
    ///
    /// If there is more then one child element of the same name (an invalid situation),
    /// only the first element is removed.
    fn clear(&self) -> Option<T> {
        let Some(to_remove) = self.get_raw() else {
            return None;
        };
        let element = self.parent();
        let mut doc = element.write_doc();
        to_remove
            .element()
            .detatch(doc.deref_mut())
            .expect("You can't detach the container element.");
        Some(T::from(to_remove))
    }

    /// Get the "raw" child [XmlElement] referenced by this [XmlChild], or `None` if the child
    /// is not present.
    fn get_raw(&self) -> Option<XmlElement> {
        let element = self.parent();
        let doc = element.read_doc();
        let (name, namespace) = (self.name(), self.namespace_url());
        let parent = element.element();
        let child = parent.find_quantified(doc.deref(), name, namespace);
        child.map(|it| XmlElement::new(element.document(), it))
    }

    /// Replace the referenced child element with a new [XmlWrapper] element and return the
    /// previous value (if any).
    ///
    /// *Warning:* This may alter the order of child elements. The updated element is typically
    /// inserted as the *last* child. Furthermore, the neither the namespace of the child,
    /// nor its name is checked against the expected values.
    ///
    /// # Document validity
    ///
    /// Obviously, this makes it possible to set the child into an invalid state.
    fn set_raw(&self, value: XmlElement) -> Option<XmlElement> {
        let element = self.parent();
        let mut doc = element.write_doc();
        let parent = element.element();

        // First, remove the existing child.
        let removed = if let Some(to_remove) = self.get_raw() {
            to_remove
                .element()
                .detatch(doc.deref_mut())
                .expect("You can't detach the container element.");
            Some(to_remove)
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
}

/// A variant of [XmlChild] that assumes the child element is a required part of the document.
pub trait RequiredXmlChild<T: XmlWrapper>: XmlChild<T> {
    /// Return the `T` wrapper for the underlying child element.
    ///
    /// # Panics
    ///
    /// Panics if the child element does not exist.
    fn get(&self) -> T {
        let Some(child) = self.get_raw() else {
            panic!("Missing child element `{}`.", self.name());
        };
        T::from(child)
    }

    /// Replaces the current value of the referenced child element with a new one. Returns the
    /// old child element.
    ///
    /// If this child is quantified with a namespace URL, then the method will automatically
    /// set the prefix of the new element to the closest viable prefix in this subtree of the
    /// document. However, the method will not create any new namespace declarations.
    ///
    /// # Panics
    ///
    /// Panics if the child element does not exist, or if the new element is not compatible
    /// with this `XmlChild` (i.e. different name). Also panics if the namespace of this child
    /// is not declared in this subtree.
    fn set(&self, value: T) -> T {
        let Some(old_child) = self.set_raw(value.into()) else {
            panic!("Missing child element `{}`.", self.name());
        };
        // If get_raw returns none, it means `value` was some invalid element
        // that does not belong here.
        let Some(current) = self.get_raw() else {
            panic!(
                "Invalid child used in document in place of `{}`.",
                self.name()
            );
        };
        let mut doc = current.write_doc();
        let quantified = current
            .element()
            .quantify_with_closest(doc.deref_mut(), self.namespace_url());
        if quantified.is_none() {
            panic!("Namespace `{}` is not decalred.", self.namespace_url());
        }
        T::from(old_child)
    }
}

/// A variant of [XmlChild] that assumes the child element is an optional part of the document.
pub trait OptionalXmlChild<T: XmlWrapper>: XmlChild<T> {
    /// Return the `T` wrapper for the underlying child element, or none if the element
    /// does not exist.
    fn get(&self) -> Option<T> {
        self.get_raw().map(|it| T::from(it))
    }

    /// Replace the current value of the referenced child element with a new one. Returns the
    /// old child element.
    fn set(&self, value: Option<T>) -> Option<T> {
        match value {
            None => self.clear(),
            Some(new_child) => {
                let old_child = self.set_raw(new_child.into());
                let Some(current) = self.get_raw() else {
                    panic!(
                        "Invalid child used in document in place of `{}`.",
                        self.name()
                    );
                };
                let mut doc = current.write_doc();
                let quantified = current
                    .element()
                    .quantify_with_closest(doc.deref_mut(), self.namespace_url());
                if quantified.is_none() {
                    panic!("Namespace `{}` is not declared.", self.namespace_url());
                }
                old_child.map(|it| T::from(it))
            }
        }
    }
}

/// Expands the capabilities of [OptionalXmlChild] when `T` implements [XmlDefault].
pub trait XmlChildDefault<T: XmlWrapper>: OptionalXmlChild<T> {
    /// The same as [XmlChild::get], but if the child does not exist, it is created using
    /// [XmlDefault::default].
    ///
    /// *Warning:* If a new element is created, it is typically inserted as the *last* child.
    fn get_or_create(&self) -> T {
        self.ensure();
        self.get().unwrap()
    }

    /// Creates the child element using [XmlDefault::default] if it does not exist.
    ///
    /// *Warning:* If a new element is created, it is typically inserted as the *last* child.
    fn ensure(&self);
}

/// Implement [XmlChildDefault] for any suitable combination of [XmlDefault] and [XmlChild] types.
impl<Element: XmlDefault, Child: OptionalXmlChild<Element>> XmlChildDefault<Element> for Child {
    fn ensure(&self) {
        if !self.is_set() {
            let default = Element::default(self.parent().document());
            self.set(Some(default));
        }
    }
}

/// Implement [XmlChildDefault] for an optional [XmlList], regardless of the inner list type.
impl<Element: XmlWrapper, Child: OptionalXmlChild<XmlList<Element>>>
    XmlChildDefault<XmlList<Element>> for Child
{
    fn ensure(&self) {
        if !self.is_set() {
            let mut document = self.parent().write_doc();
            let element = xml_doc::Element::new(document.deref_mut(), self.name());
            let element = XmlElement::new(self.parent().document(), element);
            self.set_raw(element.clone());
            let mut doc = element.write_doc();
            if element
                .element()
                .quantify_with_closest(doc.deref_mut(), self.namespace_url())
                .is_none()
            {
                panic!("Namespace `{}` is not declared.", self.namespace_url());
            }
        }
    }
}
