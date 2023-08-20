use crate::xml::{XmlDefault, XmlElement, XmlWrapper};

/// [XmlChild] implements a reference to a *required* singleton child element `T`. That is, an element
/// which is unique in its parent and represents a larger structure of type `T`.
///
/// Note that the element cannot be missing, otherwise reading will panic. Despite that, if the
/// underlying element is missing, it has to be created. This is not always possible to do automatically,
/// but there is an extension trait [XmlChildDefault] which adds this option for [XmlWrapper] types
/// that also implement [XmlDefault].
pub trait XmlChild<T: XmlWrapper> {
    /// Returns a reference to the underlying parent [XmlElement].
    fn parent(&self) -> &XmlElement;

    /// Returns `true` if the referenced child element exists
    /// (even if it is otherwise invalid).
    fn is_set(&self) -> bool;

    /// Completely remove the referenced child element.
    ///
    /// If there is more then one child element of the same name (an invalid situation),
    /// only the first element is removed.
    ///
    /// # Safety
    ///
    /// If this particular child is a required part of the structure, this may make the document
    /// invalid.
    fn clear(&self);

    /// Return the [XmlWrapper] for the underlying child element, or *panic* if the child element
    /// is not present.
    fn get(&self) -> T;

    /// Get the "raw" child [XmlElement] referenced by this [XmlChild], or *panic* if the element
    /// is not present.
    fn get_raw(&self) -> XmlElement;

    /// Replace the reference child element with a new [XmlWrapper] of type `T` and return the
    /// previous value (if any).
    ///
    /// *Warning:* This may alter the order of child elements. The updated element is typically
    /// inserted as the *last* child.
    ///
    /// If there is more then one child element of the same name (an invalid situation),
    /// only the first element is updated.
    ///
    /// The `element` argument must be in a "detached" state (i.e. with no parent). The returned
    /// value will be also in a detached state.
    fn set(&self, element: T) -> T;

    /// Provides the same functionality as [XmlChild::set], but allows using a "raw" [XmlElement].
    ///
    /// *Warning:* This may alter the order of child elements. The updated element is typically
    /// inserted as the *last* child.
    ///
    /// # Safety
    ///
    /// Obviously, this makes it possible to set the child into an invalid state.
    fn set_raw(&self, element: XmlElement) -> XmlElement;
}

/// [XmlChildOptional] is superset of [XmlChild] objects with the main difference that element may
/// be missing, in which case it *may* be created. This is not always possible to do automatically,
/// but there is an extension trait [XmlChildDefault] which adds this option for [XmlWrapper] types
/// that also implement [XmlDefault].
pub trait XmlChildOptional<T: XmlWrapper> {
    fn parent(&self) -> &XmlElement;

    fn is_set(&self) -> bool;

    fn clear(&self);

    fn get(&self) -> Option<T>;

    fn get_raw(&self) -> Option<XmlElement>;

    fn set(&self, element: T) -> Option<T>;

    fn set_raw(&self, element: XmlElement) -> Option<XmlElement>;
}

/// Expands the capabilities of [XmlChild] when `T` implements [XmlDefault].
pub trait XmlChildDefault<T: XmlWrapper>: XmlChildOptional<T> {
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
impl<Element: XmlDefault, Child: XmlChildOptional<Element>> XmlChildDefault<Element> for Child {
    fn ensure(&self) {
        if !self.is_set() {
            let default = Element::default(self.parent().document());
            self.set(default);
        }
    }
}
