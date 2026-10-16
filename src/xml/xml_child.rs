use crate::constants::namespaces::Namespace;
use crate::xml::{XmlDefault, XmlElement, XmlList, XmlWrapper};
use biodivine_lib_xml_dom::QualifiedName;

/// [XmlChild] implements a reference to a singleton child element `T`. That is, an element
/// which is unique in its parent and represents a larger structure of type `T`.
///
/// There are two variants of [XmlChild]: [OptionalXmlChild] and [RequiredXmlChild]. These
/// implement the two typical types of child elements.
///
/// ### Treatment of namespaces
///
/// Each `XmlChild` can belong to a namespace identified by a unique URL (use the empty
/// namespace for "no namespace"). The child is matched by its *expanded* name, so the actual
/// prefix used in the document does not matter.
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

    /// Returns the namespace of this child element, if any.
    fn namespace(&self) -> Option<Namespace> {
        None
    }

    /// Returns the **expanded** name of the child element.
    ///
    /// If `write_doc` is `true`, the method is allowed to ensure the conditions necessary for
    /// the returned name to be usable (in particular, to declare a namespace that the document
    /// does not declare yet). Read-only callers pass `false` and must not modify the document.
    ///
    /// The default implementation assumes the child is in no namespace. Implementations whose
    /// child belongs to a namespace override this method.
    fn qualified_name(&self, _write_doc: bool) -> Result<QualifiedName, String> {
        QualifiedName::without_namespace(self.simple_name()).map_err(|it| it.to_string())
    }

    /// Get the "raw" child [XmlElement] referenced by this [XmlChild], or `None` if the child
    /// is not present.
    fn get_raw(&self) -> Option<XmlElement> {
        let name = self.qualified_name(false).ok()?;
        self.parent()
            .child_elements()
            .into_iter()
            .find(|it| it.qualified_name() == name)
    }

    /// Replace the referenced child element with a new [XmlElement] and return the
    /// previous value (if any).
    ///
    /// If the corresponding child element already exists, it is replaced in place. Otherwise,
    /// the element is appended as the new last child.
    ///
    /// # Panics
    ///
    ///  - The inserted element must have the correct expanded name.
    ///  - Can panic if the underlying document rejects the insertion.
    fn set_raw(&self, value: XmlElement) -> Option<XmlElement> {
        let expected = self.qualified_name(true).unwrap_or_else(|_| {
            panic!(
                "Cannot ensure valid namespace for `{}`.",
                self.simple_name()
            )
        });

        if value.qualified_name() != expected {
            panic!(
                "Cannot set XML child `{}` to value `{}`.",
                expected,
                value.full_name(),
            );
        }

        match self.get_raw() {
            Some(old) => {
                // Replacing a child with itself is a no-op; `replace_with` would otherwise
                // try to insert a node into its own position.
                if old.element() != value.element() {
                    old.element().node().replace_with(value.element().node());
                }
                Some(old)
            }
            None => {
                self.parent().element().append_child(value.element().node());
                None
            }
        }
    }

    /// Completely remove the referenced child element and return it (if it is present).
    fn clear_raw(&self) -> Option<XmlElement> {
        let to_remove = self.get_raw()?;
        to_remove.element().detach();
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

        // The cast is ok because `get_raw` resolves the child by its expanded name, which is
        // exactly the name expected by this `XmlChild`.
        T::from(child)
    }

    /// Replaces the current value of the referenced child element with a new one. Returns the
    /// old child element.
    ///
    /// # Panics
    ///
    /// The method panics if:
    ///  - The child does not exist (there is no old value to return).
    ///  - The new value is not compatible with this child (different expanded name).
    fn set(&self, value: T) -> T {
        let Some(old) = self.set_raw(value.into()) else {
            panic!("Missing child element `{}`.", self.simple_name());
        };

        // See [RequiredXmlChild::get].
        T::from(old)
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
        // See [RequiredXmlChild::get].
        self.get_raw().map(T::from)
    }

    /// Replace the current value of the referenced child element with a new one. Returns the
    /// old child element, if any.
    ///
    /// # Panics
    ///
    /// The method panics if the new value is not compatible with this child (different
    /// expanded name).
    fn set(&self, value: T) -> Option<T> {
        // See [RequiredXmlChild::get].
        self.set_raw(value.into()).map(T::from)
    }

    /// Completely remove the referenced child element and return it (if it is present).
    fn clear(&self) -> Option<T> {
        // See [RequiredXmlChild::get].
        self.clear_raw().map(T::from)
    }
}

/// Expands the capabilities of [OptionalXmlChild] to allow automatic creation
/// of the child elements.
pub trait XmlChildDefault<T: XmlWrapper>: OptionalXmlChild<T> {
    /// The same as [OptionalXmlChild::get], but if the child does not exist, it is created.
    ///
    /// *Warning:* If a new element is created, it is inserted as the *last* child.
    fn get_or_create(&self) -> T {
        self.ensure();
        self.get().unwrap()
    }

    /// If the underlying XML child element does not exist, it is created.
    ///
    /// *Warning:* If a new element is created, it is inserted as the *last* child.
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
            let parent = self.parent();
            // We want to call `qualified_name` in write mode because it ensures the relevant
            // package is declared if that is necessary.
            let name = self.qualified_name(true).unwrap_or_else(|_| {
                panic!(
                    "Cannot ensure valid namespace for `{}`.",
                    self.simple_name()
                )
            });
            let document = parent.document();
            let child = XmlElement::new(document.create_element(name.clone()));
            if let Some(namespace) = name.namespace() {
                child.ensure_namespace(namespace);
            }
            self.set_raw(child);
        }
    }
}
