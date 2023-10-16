use crate::xml::{
    OptionalDynamicChild, OptionalDynamicProperty, RequiredDynamicChild, RequiredDynamicProperty,
    XmlDocument, XmlElement, XmlPropertyType,
};
use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use xml_doc::{Document, Element};

/// [XmlWrapper] is a trait implemented by all types that can behave as an [XmlElement]
/// (including [XmlElement] itself). In other words, [XmlWrapper] is implemented by all types
/// that provide some "type safe" view of an underlying XML element.
///
/// However, note that the aforementioned type safety is only checked when the properties
/// or children of the element are actually read (or written). There is no explicit check
/// for the validity of the XML structure at the time when [XmlWrapper] is created. In other
/// words, any XML element *can* be interpreted as any [XmlWrapper] instance, and it is then
/// up to the [XmlWrapper] instance to generate errors if invalid properties are read or written.
/// In fact, the trait actually requires infallible [From] and [Into] conversions
/// w.r.t [XmlElement].
///
/// The trait also provides basic utility functions over the underlying [XmlElement] instance
/// so that one does not have to constantly call [XmlWrapper::as_xml] to perform any XMl operation.
pub trait XmlWrapper: Into<XmlElement> {
    /// Obtain a reference to the underlying [XmlElement].
    fn as_xml(&self) -> &XmlElement;

    /*
    /// Perform a checked conversion of arbitrary [XmlWrapper] `T` into `Self`.
    ///
    /// Note that any checks performed by the conversion should be contained to the underlying
    /// XML element and nothing else. For example, this should not check that the children
    /// are also valid.
    ///
    /// The reason why we do this instead of using `TryFrom` is that we need error messages that
    /// are readable, but specifying that every error should implement `Display` is a huge mess.
    fn checked_cast<T: XmlWrapper>(element: T) -> Result<Self, String>;
     */

    /// Performs an unsafe conversion from an arbitrary [XmlWrapper] `T` into `Self`.
    /// The conversion always succeeds regardless of the structure of `element`, hence it is
    /// considered unsafe.
    ///
    /// This is more or less equivalent to requiring `From<XmlElement>` implementation for
    /// every [XmlWrapper], but it makes it explicit that the conversion is unsafe.
    ///
    /// # Safety
    ///
    /// Obviously, this operation is only safe if you know that the XML element in question
    /// can be used as a valid value of type `T`.
    unsafe fn unchecked_cast<T: XmlWrapper>(element: T) -> Self;

    /*
    /// Perform a "cast" to type `T`. The conversion is performed through the `T::checked_from`
    /// method. Panics if the cast is not successful.
    ///
    /// In other words, this is equivalent to `T::checked_from(x).unwrap()`, but provides a nicer
    /// error message.
    fn cast<T: XmlWrapper>(self) -> T {
        match T::checked_cast(self) {
            Ok(value) => value,
            Err(error) => {
                panic!("Cannot cast tag `{}` to `{}` wrapper: {}.",
                    self.name(),
                    std::any::type_name::<T>(),
                    error
                );
            }
        }
    }
     */

    /// Obtain a (counted) reference to the underlying [XmlDocument].
    fn document(&self) -> XmlDocument {
        self.as_xml().document.clone()
    }

    /// Get the [Element] instance for the underlying [XmlElement].
    fn raw_element(&self) -> Element {
        self.as_xml().element
    }

    /// Obtain a read-only reference to the underlying [Document].
    fn read_doc(&self) -> RwLockReadGuard<Document> {
        // Error handling note: In general, lock access will fail only when some other part
        // of the program performed an incorrect unsafe action (e.g. double release of the
        // same lock guard). As such, it is generally ok to panic here, because at that point
        // the whole document might be corrupted and we have no way to recover.
        self.as_xml()
            .document
            .read()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Obtain a writeable reference to the underlying [Document].
    fn write_doc(&self) -> RwLockWriteGuard<Document> {
        // See [Self::read_doc] for error handling notes.
        self.as_xml()
            .document
            .write()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Returns the name of the XML tag referenced within this [XmlWrapper].
    ///
    /// Note that for most implementations of [XmlWrapper], this value will be constant.
    /// However, this is not strictly required by [XmlWrapper], so there can be implementations
    /// where this value changes depending on context.
    fn name(&self) -> String {
        let doc = self.read_doc();
        self.raw_element().name(doc.deref()).to_string()
    }

    /// Returns the namespace URL of the XML tag referenced within this [XmlWrapper].
    ///
    /// Note that for most implementations of [XmlWrapper], this value will be constant.
    /// However, this is not strictly required by [XmlWrapper], so there can be implementations
    /// where this value changes depending on context.
    fn namespace_url(&self) -> String {
        let doc = self.read_doc();
        self.raw_element()
            .namespace(doc.deref())
            .unwrap_or("")
            .to_string()
    }

    /// Get a reference to a specific **required** [XmlProperty] of this XML element.
    ///
    /// Note that individual [XmlWrapper] implementations should provide type safe access
    /// to their known/required properties through specialised [XmlProperty] implementations
    /// instead of relying on [RequiredDynamicProperty].
    fn required_property<T: XmlPropertyType>(&self, name: &str) -> RequiredDynamicProperty<T> {
        RequiredDynamicProperty::new(self.as_xml(), name)
    }

    /// Get a reference to a specific **optional** [XmlProperty] of this XML element.
    ///
    /// Also see [Self::required_property].
    fn optional_property<T: XmlPropertyType>(&self, name: &str) -> OptionalDynamicProperty<T> {
        OptionalDynamicProperty::new(self.as_xml(), name)
    }

    /// Get a reference to a specific **optional** [XmlChild] of this XML element.
    ///
    /// Also see [Self::required_property].
    fn optional_child<T: XmlWrapper>(
        &self,
        name: &str,
        namespace_url: &str,
    ) -> OptionalDynamicChild<T> {
        OptionalDynamicChild::new(self.as_xml(), name, namespace_url)
    }

    /// Get a reference to a specific **required** [XmlChild] of this XML element.
    ///
    /// Also see [Self::required_property].
    fn required_child<T: XmlWrapper>(
        &self,
        name: &str,
        namespace_url: &str,
    ) -> RequiredDynamicChild<T> {
        RequiredDynamicChild::new(self.as_xml(), name, namespace_url)
    }

    /// Detach this [XmlWrapper] from its current parent while maintaining the necessary
    /// namespace declarations that make the XML sub-tree valid.
    ///
    /// Specifically, this method will:
    ///  - Scan the whole XML sub-tree for relevant namespace prefixes
    ///    (i.e. those not declared in the sub-tree itself).
    ///  - Copy the namespace declarations into the sub-tree root.
    ///  - Detach the element.
    ///  - Due to the copied declarations, all prefixes are still valid,
    ///    even in this detached state.
    ///
    /// Note that this also applies to the "default" namespace: if an empty prefix is used and
    /// not declared, the method will add `xmlns=""` to the detached tag in order to reset
    /// the default namespace in the root of the detached sub-tree.
    ///
    /// The method fails if the element is already detached, or when the element is the
    /// document "container" element (which is, in theory, always detached).
    fn try_detach(&self) -> Result<(), String> {
        // Note that we can't use methods like `Self::name` because they would need to lock
        // the document and we already have it locked.
        let element = self.raw_element();
        let mut doc = self.write_doc();
        if element.parent(doc.deref()).is_none() {
            return Err(format!(
                "Cannot detach `{}`. Already detached.",
                element.name(doc.deref())
            ));
        }
        let retain = element.collect_external_namespace_decls(doc.deref());
        if let Err(e) = element.detatch(doc.deref_mut()) {
            return Err(format!(
                "Cannot detach `{}`. Internal XML error: `{}`.",
                element.name(doc.deref()),
                e
            ));
        }
        let current = element.mut_namespace_decls(doc.deref_mut());
        current.extend(retain);

        Ok(())
    }

    /// Returns `true` if this element is in a detached state (i.e. it has no parent).
    fn is_detached(&self) -> bool {
        let element = self.raw_element();
        let doc = self.read_doc();
        element.parent(doc.deref()).is_none()
    }

    /// Try to attach this [XmlWrapper] into the given `parent` [XmlWrapper] as a new child at
    /// the given `position`. If `position` is not given, the child is inserted as the
    /// last element. The method maintains the namespace declarations
    /// necessary for the child sub-tree.
    ///
    /// Errors:
    ///  - If `self` is not detached or is the container element.
    ///  - If `position > parent.children().len()`.
    ///
    /// The method takes all namespaces which are declared directly on `self`, and tries
    /// to propagate them to the root element reachable from parent (i.e. either a document
    /// root, or a root of a detached sub-tree). Specifically:
    ///  - The declaration of a default namespace cannot be propagated. However, it can be removed
    ///    if the root declares the same namespace.
    ///  - The prefixes which are already declared with matching namespace URLs are removed.
    ///  - The prefixes which are not declared anywhere on the path to root
    ///    can be propagated to root.
    ///  - Other namespaces cannot be propagated, because their prefix is already declared,
    ///    but with another URL.
    ///
    /// Note that if the attached sub-tree has no default namespace, then it should have `xmlns=""`
    /// set. This then ensures the namespace information correctly propagates within the document.
    fn try_attach_at<W: XmlWrapper>(
        &self,
        parent: &W,
        position: Option<usize>,
    ) -> Result<(), String> {
        // !! See `try_detach` note about deadlocks and self methods. !!
        let element = self.raw_element();
        let parent_element = parent.raw_element();
        let mut doc = self.write_doc();

        // First, check that everything is ok.

        if element.parent(doc.deref()).is_some() {
            return Err(format!(
                "Cannot attach `{}`. Not detached.",
                element.name(doc.deref())
            ));
        }
        let child_count = parent_element.children(doc.deref()).len();
        let position = position.unwrap_or(child_count);
        if position > child_count {
            return Err(format!(
                "Cannot attach `{}`. Invalid position `{} > {}`.",
                element.name(doc.deref()),
                position,
                child_count
            ));
        }

        // Now we can actually attach the child.

        if let Err(e) = parent_element.insert_child(doc.deref_mut(), position, element.as_node()) {
            return Err(format!(
                "Cannot detach `{}`. Internal XML error: `{}`.",
                element.name(doc.deref()),
                e
            ));
        }

        // And finally, we can clean up the namespaces.

        let applicable_namespaces = parent_element.collect_applicable_namespace_decls(doc.deref());
        let child_namespaces = element.namespace_decls(doc.deref()).clone();
        let top_element = parent_element.top_parent(doc.deref());
        for (prefix, namespace) in child_namespaces {
            if prefix.is_empty() {
                // Default namespace with empty prefix cannot be propagated.
                if let Some(default) = applicable_namespaces.get(&prefix) {
                    if *default == namespace {
                        // However, if the same default namespace already exists, we can remove it.
                        element
                            .mut_namespace_decls(doc.deref_mut())
                            .remove(prefix.as_str());
                    }
                } else {
                    // Or, if the default namespace does not exist, we can remove if as well,
                    // because it is implied.
                    element
                        .mut_namespace_decls(doc.deref_mut())
                        .remove(prefix.as_str());
                }
                continue;
            }
            if let Some(declared) = applicable_namespaces.get(&prefix) {
                if *declared == namespace {
                    // The prefix is already declared with the same URL. Hence we can remove
                    // the declaration on the child element because it is redundant.
                    element
                        .mut_namespace_decls(doc.deref_mut())
                        .remove(prefix.as_str());
                } else {
                    // Otherwise we must keep the declaration on the child element, because
                    // the prefix is already used for other namespaces elsewhere in the document.
                    continue;
                }
            } else {
                // The prefix is not used in this document yet. We can re-declare it in the
                // root to make it easier to work with.
                element
                    .mut_namespace_decls(doc.deref_mut())
                    .remove(prefix.as_str());
                top_element.set_namespace_decl(doc.deref_mut(), prefix, namespace);
            }
        }

        Ok(())
    }
}

/// [XmlDefault] extends the functionality of [XmlWrapper] by providing a method that can build
/// a "default" value of `Self` in the given [XmlDocument].
///
/// The resulting element should be in a "detached" state, meaning it has no parent.
///
/// Ideally, the result should represent a valid value of type `Self`. However, it does not have
/// to take into account the global state of the document (e.g. if there is an ID, it may not be
/// unique).
pub trait XmlDefault: XmlWrapper {
    /// Construct a "default" value of this type in the provided [XmlDocument].
    fn default(document: XmlDocument) -> Self;
}
