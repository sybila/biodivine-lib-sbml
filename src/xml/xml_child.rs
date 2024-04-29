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
/// is truly a singleton. Undefined behaviour can occur if this is not the case. Ideally,
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
    fn name(&self) -> &str;

    /// Returns the namespace URL of this child.
    ///
    /// The url can be empty, in which case it corresponds to the "default" empty namespace
    /// (i.e. the namespace in which tags reside when there is no default namespace declared).
    /// Just as the name, this value is considered immutable and all XML elements that appear
    /// in the `get`/`set` methods must use this namespace.
    fn namespace_url(&self) -> &str;

    /// Get the "raw" child [XmlElement] referenced by this [XmlChild], or `None` if the child
    /// is not present.
    fn get_raw(&self) -> Option<XmlElement> {
        let element = self.parent();
        let doc = element.read_doc();
        let (name, namespace) = (self.name(), self.namespace_url());
        let parent = element.raw_element();
        let child = parent.find_quantified(doc.deref(), name, namespace);
        child.map(|it| XmlElement::new_raw(element.document(), it))
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
        let element = self.parent();
        let parent = element.raw_element();

        // First, check that the new value has the correct name and namespace.
        if value.tag_name() != self.name() || value.namespace_url() != self.namespace_url() {
            panic!(
                "Cannot set XML child `({},{})` to value `({},{})`.",
                self.name(),
                self.namespace_url(),
                value.tag_name(),
                value.namespace_url(),
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
                // The element should be always safe to detach assuming the document is in
                // a consistent state.
                unreachable!()
            }
            (Some(to_remove), index)
        } else {
            (None, None)
        };

        // Now, push the new child and check that the result is ok.
        if let Err(e) = value.try_attach_at(element, index) {
            panic!("Cannot set value of child `{}`: {}", self.name(), e);
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
        let Some(to_remove) = self.get_raw() else {
            return None;
        };
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
            panic!("Missing child element `{}`.", self.name());
        };

        // The cast is ok, because the `get_raw` method only succeeds if the quantified name
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
            panic!("Missing child element `{}`.", self.name());
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
    /// Return the `T` wrapper for the underlying child element, or none if the element
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
    /// The same as [XmlChild::get], but if the child does not exist, it is created.
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
///
/// This approach assumes the namespace of the [XmlChild] is already declared. If it is not
/// declared somewhere in the document, the empty prefix is used to declare it.
impl<Element: XmlWrapper, Child: OptionalXmlChild<XmlList<Element>>>
    XmlChildDefault<XmlList<Element>> for Child
{
    fn ensure(&self) {
        if self.get_raw().is_none() {
            let url = self.namespace_url();
            let prefix: String = {
                let doc = self.parent().read_doc();
                self.parent()
                    .element
                    .closest_prefix(doc.deref(), url)
                    .unwrap_or("")
                    .to_string()
            };
            let list_element = XmlElement::new_quantified(
                self.parent().document(),
                self.name(),
                (prefix.as_str(), url),
            );
            self.set_raw(list_element);
        }
    }
}
