use crate::constants::namespaces::{Namespace, NS_EMPTY};
use crate::xml::{XmlDefault, XmlElement, XmlList, XmlWrapper};
use std::ops::Deref;

/// [XmlChild] implements a reference to a singleton child element `T`. That is, an element
/// which is unique in its parent and represents a larger structure of type `T`.
///
/// There are two variants of [XmlChild]: [OptionalXmlChild] and [RequiredXmlChild]. These
/// implement the two typical types of child elements.
///
/// ### Treatment of namespaces
///
/// Each `XmlChild` can belong to a namespace identified by a unique URL (use empty string for
/// "no namespace"). The assumption is that this URL is resolved into a correct namespace prefix
/// dynamically, based on the context in which the `XmlChild` resides.
///
/// Note that implementations of [XmlChild] do not update the namespace declarations on the
/// elements in any way (in particular, we don't add any specific namespace declaration or prefix).
/// However, they do use [XmlWrapper::try_detach] and [XmlWrapper::try_attach_at] to consistently
/// maintain namespaces between different contexts.
///
/// ### On singleton validation
///
/// *Warning:* At the moment, [XmlChild] implementations do not check that the child element
/// is truly a singleton. Undefined behavior can occur if this is not the case. Ideally,
/// this condition should be checked by additional document-wide validation steps.
pub trait XmlChild<T: XmlWrapper> {
    /// Returns a reference to the underlying parent [XmlElement].
    ///
    /// It is expected that this reference is immutable. An `XmlChild` instance is forever
    /// bound to a single parent element.
    fn parent(&self) -> &XmlElement;

    /// Returns the name of the corresponding child tag.
    ///
    /// It is expected that this name is immutable. That is, an `XmlChild` instance is associated
    /// with a specific tag name, and this name must not change. It is also required that all
    /// XML elements that appear in `get`/`set` methods use this tag name.
    fn simple_name(&self) -> &str;

    /// Returns the namespace of this child element, if any. This is used to make sure
    /// any newly created child tags have the proper namespace declared on them.
    fn namespace(&self) -> Option<Namespace> {
        None
    }

    /// Returns the **fully quantified** name of the underlying XML tag, including
    /// namespace prefix if relevant. Can return an error at runtime if there is some problem
    /// with the construction of the quantified name (e.g., the namespace is not declared, or
    /// not declared correctly).
    ///
    /// This name can (and probably should be) computed dynamically at runtime for properties
    /// that belong to a specific non-default namespace, as the prefix can change depending
    /// on the position of the property in the document.
    ///
    /// If `write_doc` is set to `true`, it indicates to the method that it can try to
    /// ensure necessary conditions for the quantified name to be valid (e.g., create a namespace
    /// declaration). The exact conditions as to when this is valid can vary depending on the
    /// implementation. In general, methods that only read the document should not allow
    /// any modification. Meanwhile, methods that write values to the document can set this to
    /// true to indicate that "fixing" the document into a consistent state is allowed.
    ///
    /// The default implementation for this method simply returns [XmlChild::simple_name]
    /// (i.e., it assumes the attribute is in the default empty namespace). Please override this
    /// in cases where the property can depend on XML namespaces.
    ///
    /// The same as [crate::xml::XmlProperty::quantified_name].
    fn quantified_name(&self, _write_doc: bool) -> Result<String, String> {
        Ok(self.simple_name().to_string())
    }

    /// Get the "raw" child [XmlElement] referenced by this [XmlChild], or `None` if the child
    /// is not present.
    fn get_raw(&self) -> Option<XmlElement> {
        let Ok(full_name) = self.quantified_name(false) else {
            return None;
        };

        let element = self.parent();
        let doc = element.read_doc();
        for child in element.raw_element().child_elements(&doc) {
            if child.full_name(&doc) == full_name {
                return Some(XmlElement::new_raw(element.document(), child));
            }
        }
        None
    }

    /// Replace the referenced child element with a new [XmlWrapper] element and return the
    /// previous value (if any).
    ///
    /// If the corresponding child element already exists, it is replaced. Otherwise, the element
    /// is inserted as a new last child.
    ///
    /// # Panics
    ///
    ///  - The inserted element must have the correct tag name and namespace url.
    ///  - The inserted element must be in a detached state.
    ///  - Can panic if the old child cannot be detached, but this should be unreachable.
    fn set_raw(&self, value: XmlElement) -> Option<XmlElement> {
        let Ok(full_name) = self.quantified_name(true) else {
            panic!(
                "Cannot ensure valid namespace for `{}`.",
                self.simple_name()
            );
        };

        let element = self.parent();
        let parent = element.raw_element();

        // First, check that the new value has the correct name and namespace.
        let expected_namespace = self.namespace().unwrap_or(NS_EMPTY);
        if value.tag_name() != self.simple_name() || value.namespace_url() != expected_namespace.1 {
            panic!(
                "Cannot set XML child `{}` to value `{}`.",
                full_name,
                value.full_name(),
            )
        }

        // Then, remove the existing child.
        let (removed, index) = if let Some(to_remove) = self.get_raw() {
            let index = {
                let doc = self.parent().read_doc();
                parent
                    .child_elements(doc.deref())
                    .into_iter()
                    .position(|e| e == to_remove.raw_element())
            };
            if to_remove.try_detach().is_err() {
                // The element should always be safe to detach assuming the document is in
                // a consistent state.
                unreachable!()
            }
            (Some(to_remove), index)
        } else {
            (None, None)
        };

        // Now, push the new child and check that the result is ok.
        if let Err(e) = value.try_attach_at(element, index) {
            panic!("Cannot set value of child `{}`: {}", self.simple_name(), e);
        }

        // Return the old child.
        removed
    }

    /// Completely remove the referenced child element and return it (if it is present).
    ///
    /// # Panics
    ///
    /// Can panic if the child cannot be detached (should not happen in normal situations).
    fn clear_raw(&self) -> Option<XmlElement> {
        let to_remove = self.get_raw()?;
        if let Err(e) = to_remove.try_detach() {
            panic!("{}", e);
        }

        // See [RequiredXmlChild::get].
        Some(to_remove)
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
            panic!("Missing child element `{}`.", self.simple_name());
        };

        // The cast is ok because the `get_raw` method only succeeds if the quantified name
        // of the returned element matches the quantified name specified by this XmlChild.
        unsafe { T::unchecked_cast(child) }
    }

    /// Replaces the current value of the referenced child element with a new one. Returns the
    /// old child element.
    ///
    /// The method updates the namespace declarations based on
    /// the rules of [XmlWrapper::try_attach_at].
    ///
    /// # Panics
    ///
    /// The method panics if:
    ///  - The child does not exist (there is no old value to return).
    ///  - The new value is not compatible with this child (different name or namespace url).
    ///  - The new value is not detached.
    ///  - Can panic if the old child cannot be detached, but this should be unreachable.
    fn set(&self, value: T) -> T {
        let Some(old) = self.set_raw(value.into()) else {
            panic!("Missing child element `{}`.", self.simple_name());
        };

        // See [RequiredXmlChild::get].
        unsafe { T::unchecked_cast(old) }
    }
}

/// A variant of [XmlChild] that assumes the child element is an optional part of the document.
pub trait OptionalXmlChild<T: XmlWrapper>: XmlChild<T> {
    /// True if the value of this optional child is set.
    fn is_set(&self) -> bool {
        self.get_raw().is_some()
    }
    /// Return the `T` wrapper for the underlying child element or none if the element
    /// does not exist.
    fn get(&self) -> Option<T> {
        self.get_raw().map(|it| {
            // See [RequiredXmlChild::get].
            unsafe { T::unchecked_cast(it) }
        })
    }

    /// Replace the current value of the referenced child element with a new one. Returns the
    /// old child element, if any.
    ///
    /// # Panics
    ///
    /// The method panics if:
    ///  - The new value is not compatible with this child (different name or namespace url).
    ///  - The new value is not detached.
    ///  - Can panic if the old child cannot be detached, but this should be unreachable.
    fn set(&self, value: T) -> Option<T> {
        // See [RequiredXmlChild::get].
        self.set_raw(value.into())
            .map(|it| unsafe { T::unchecked_cast(it) })
    }

    /// Completely remove the referenced child element and return it (if it is present).
    ///
    /// # Panics
    ///
    ///  - Can panic if the old child cannot be detached, but this should be unreachable.
    fn clear(&self) -> Option<T> {
        // See [RequiredXmlChild::get].
        self.clear_raw().map(|it| unsafe { T::unchecked_cast(it) })
    }
}

/// Expands the capabilities of [OptionalXmlChild] to allow automatic creation
/// of the child elements.
pub trait XmlChildDefault<T: XmlWrapper>: OptionalXmlChild<T> {
    /// The same as [OptionalXmlChild::get], but if the child does not exist, it is created.
    ///
    /// *Warning:* If a new element is created, it is typically inserted as the *last* child.
    fn get_or_create(&self) -> T {
        self.ensure();
        self.get().unwrap()
    }

    /// If the underlying XML child element does not exist, it is created.
    ///
    /// *Warning:* If a new element is created, it is typically inserted as the *last* child.
    fn ensure(&self);
}

/// Implement [XmlChildDefault] for any suitable combination of [XmlDefault] and [XmlChild] types.
impl<Element: XmlDefault, Child: OptionalXmlChild<Element>> XmlChildDefault<Element> for Child {
    fn ensure(&self) {
        if self.get_raw().is_none() {
            let default = Element::default(self.parent().document());
            self.set(default);
        }
    }
}

/// Implement [XmlChildDefault] for an optional [XmlList], regardless of the inner list type.
impl<Element: XmlWrapper, Child: OptionalXmlChild<XmlList<Element>>>
    XmlChildDefault<XmlList<Element>> for Child
{
    fn ensure(&self) {
        if self.get_raw().is_none() {
            // We want to call "quantified name" in both situations because it ensures the
            // relevant package is correctly declared if it is necessary.
            let Ok(full_name) = self.quantified_name(true) else {
                panic!(
                    "Cannot ensure valid namespace for `{}`.",
                    self.simple_name()
                );
            };
            let child_element = if let Some(namespace) = self.namespace() {
                XmlElement::new_quantified(self.parent().document(), self.simple_name(), namespace)
            } else {
                let child = {
                    let mut doc = self.parent().write_doc();
                    biodivine_xml_doc::Element::new(&mut doc, full_name)
                };
                XmlElement::new_raw(self.parent().document(), child)
            };
            self.set_raw(child_element);
        }
    }
}
