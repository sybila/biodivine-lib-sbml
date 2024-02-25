use crate::xml::{
    OptionalDynamicChild, OptionalDynamicProperty, RequiredDynamicChild, RequiredDynamicProperty,
    XmlDocument, XmlElement, XmlPropertyType,
};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use xml_doc::{Document, Element};

/// [XmlWrapper] is a trait implemented by all types that can behave as an [XmlElement]
/// (including [XmlElement] itself). In other words, instances of [XmlWrapper] provide
/// some "type safe" view of the underlying XML element.
///
/// However, the aforementioned type safety is only checked when the properties
/// or children of the element are actually read (or written). There is no explicit check
/// for the validity of the XML structure at the time when [XmlWrapper] is created. In theory,
/// any XML element *can* be interpreted as any [XmlWrapper] instance and it is then
/// up to the [XmlWrapper] instance to generate errors if invalid properties are read or written.
///
/// To convert [XmlWrapper] to [XmlElement], you can use `From`/`Into`. For the reverse conversion,
/// you can use the [Self::unchecked_cast] method, but this one is marked `unsafe` to make it
/// clear the validity of the result is not checked in any formal way.
///
/// [XmlWrapper] can safely manipulate elements that use namespaces. More details about this
/// behaviour are given in [XmlWrapper::try_detach] and [XmlWrapper::try_attach_at].
pub trait XmlWrapper: Into<XmlElement> {
    /// Obtain a reference to the underlying [XmlElement].
    fn xml_element(&self) -> &XmlElement;

    /// Performs an unsafe conversion from an arbitrary [XmlWrapper] type `T` into `Self`.
    /// The conversion does not check the validity of the `element` in any way, hence it is
    /// considered `unsafe`.
    ///
    /// This is more or less equivalent to requiring `From<XmlElement>` implementation for
    /// every [XmlWrapper]. However, this way we can make the method explicitly `unsafe`
    /// to indicate that no conversion checks are taking place.
    ///
    /// # Safety
    ///
    /// This operation cannot generate true "unsafe" undefined behaviour. But if an illegal
    /// cast is performed, the resulting object can exhibit behaviour which does not follow
    /// the contracts of the [XmlWrapper] methods.
    unsafe fn unchecked_cast<T: XmlWrapper>(element: T) -> Self;

    /// Obtain a (counted) reference to the underlying [XmlDocument].
    fn document(&self) -> XmlDocument {
        self.xml_element().document.clone()
    }

    /// Get the [Element] instance for the underlying [XmlElement].
    ///
    /// Note that directly interfacing with `xml-doc` elements is strongly discouraged,
    /// because it is easy to unintentionally perform operations that can break the contracts
    /// of the [XmlWrapper] methods.
    fn raw_element(&self) -> Element {
        self.xml_element().element
    }

    /// Obtain a read-only reference to the underlying [Document].
    ///
    /// If another reference obtained through [Self::write_doc] is still
    /// alive (e.g. due to recursion), calling this method will cause a deadlock.
    /// Multiple [Self::read_doc] references are fine.
    fn read_doc(&self) -> RwLockReadGuard<Document> {
        // Error handling note: In general, lock access will fail only when some other part
        // of the program performed an incorrect unsafe action (e.g. double release of the
        // same lock guard). As such, it is generally ok to panic here, because at that point
        // the whole document might be corrupted and we have no way to recover.
        self.xml_element()
            .document
            .read()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Obtain a writeable reference to the underlying [Document].
    ///
    /// If another reference obtained through [Self::read_doc] or [Self::write_doc] is still
    /// alive (e.g. due to recursion), calling this method will cause a deadlock.
    fn write_doc(&self) -> RwLockWriteGuard<Document> {
        // See [Self::read_doc] for error handling notes.
        self.xml_element()
            .document
            .write()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Returns the name of the XML tag referenced within this [XmlWrapper].
    ///
    /// Note that for most implementations of [XmlWrapper], this value will be a compile time
    /// constant. However, this is not strictly required by [XmlWrapper], so there can be
    /// implementations where this value changes depending on context. For example, [XmlList]
    /// implements [XmlWrapper], but only determines its name at runtime.
    fn tag_name(&self) -> String {
        let doc = self.read_doc();
        self.raw_element().name(doc.deref()).to_string()
    }

    /// Returns the namespace URL of the XML tag referenced within this [XmlWrapper].
    ///
    /// Same notes about value immutability as for [Self::name] apply.
    fn namespace_url(&self) -> String {
        let doc = self.read_doc();
        self.raw_element()
            .namespace(doc.deref())
            .unwrap_or("")
            .to_string()
    }

    /// Returns the map of attributes as a collection of key-value pairs **<full_name:value>**
    /// referenced within this [XmlWrapper].
    ///
    /// Note that full_name generally consists of namespace prefix and actual name in following format: **prefix:name**.
    fn attributes(&self) -> HashMap<String, String> {
        let doc = self.read_doc();
        self.raw_element().attributes(doc.deref()).clone()
    }

    /// Returns true if this [XmlWrapper] instance has an attribute of the given name.
    fn has_attribute(&self, name: &str) -> bool {
        let doc = self.read_doc();
        self.raw_element().attribute(doc.deref(), name).is_some()
    }

    /// Return the raw value of the specified attribute, if it is defined.
    fn get_attribute(&self, name: &str) -> Option<String> {
        let doc = self.read_doc();
        self.raw_element()
            .attribute(doc.deref(), name)
            .map(|it| it.to_string())
    }

    /// Return the text content of this element and all its children.
    fn text_content(&self) -> String {
        let doc = self.read_doc();
        self.raw_element().text_content(doc.deref())
    }

    /// Return the parent element of this [XmlWrapper] instance, if any.
    fn parent(&self) -> Option<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .parent(doc.deref())
            .map(|it| XmlElement::new_raw(self.document(), it))
    }

    /// Returns the vector of children referenced within this [XmlWrapper] as a collection
    /// of [Element] objects. This method skips any child nodes that are not elements (such as
    /// text or comments).
    fn child_elements(&self) -> Vec<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .child_elements(doc.deref())
            .into_iter()
            .map(|it| XmlElement::new_raw(self.document(), it))
            .collect()
    }

    /// Get the `i-th` child element of this XML element. This operation ignores comments
    /// or text content and only considers "true" child elements.
    fn get_child_at(&self, index: usize) -> Option<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .children(doc.deref())
            .iter()
            .filter_map(|it| it.as_element())
            .skip(index)
            .map(|it| XmlElement::new_raw(self.document(), it))
            .next()
    }

    /// Version of [Self::child_elements] with additional filtering function applied to the
    /// output vector.
    fn child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        predicate: P,
    ) -> Vec<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .child_elements(doc.deref())
            .into_iter()
            .map(|it| XmlElement::new_raw(self.document(), it))
            .filter(predicate)
            .collect()
    }

    /// Version of [Self::child_elements] that recursively traverses all child nodes, not just
    /// the immediate descendants.
    fn recursive_child_elements(&self) -> Vec<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .child_elements_recursive(doc.deref())
            .into_iter()
            .map(|it| XmlElement::new_raw(self.document(), it))
            .collect()
    }

    /// Version of [Self::recursive_child_elements] with additional filtering function applied
    /// to the output vector.
    fn recursive_child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        predicate: P,
    ) -> Vec<XmlElement> {
        let doc = self.read_doc();
        self.raw_element()
            .child_elements_recursive(doc.deref())
            .into_iter()
            .map(|it| XmlElement::new_raw(self.document(), it))
            .filter(predicate)
            .collect()
    }

    /// Returns the vector of names of children referenced within this [XmlWrapper].
    fn children_names(&self) -> Vec<&str> {
        unimplemented!();
        // let doc = self.read_doc();
        // self.raw_element()
        //     .children(doc.deref())
        //     .iter()
        //     .map(|node| node.as_element().unwrap().full_name(doc.deref())) // TODO: may outlive borrowed value `doc`. How to fix?
        //     .collect()
    }

    /// Get a reference to a specific **required** [XmlProperty] of this XML element.
    ///
    /// # Safety
    ///
    /// Note that individual [XmlWrapper] implementations should provide type safe access
    /// to their known/required properties through specialised [XmlProperty] implementations
    /// instead of relying on [RequiredDynamicProperty]. Using this method is to some extent
    /// equivalent to using [Self::unchecked_cast], because the validity of the requested
    /// property is not verified in any way.
    fn required_property<T: XmlPropertyType>(&self, name: &str) -> RequiredDynamicProperty<T> {
        RequiredDynamicProperty::new(self.xml_element(), name)
    }

    /// Get a reference to a specific **optional** [XmlProperty] of this XML element.
    ///
    /// Also see notes on safety in [Self::required_property].
    fn optional_property<T: XmlPropertyType>(&self, name: &str) -> OptionalDynamicProperty<T> {
        OptionalDynamicProperty::new(self.xml_element(), name)
    }

    /// Get a reference to a specific **optional** [XmlChild] of this XML element.
    ///
    /// Also see notes on safety in [Self::required_property].
    fn optional_child<T: XmlWrapper>(
        &self,
        name: &str,
        namespace_url: &str,
    ) -> OptionalDynamicChild<T> {
        OptionalDynamicChild::new(self.xml_element(), name, namespace_url)
    }

    /// Get a reference to a specific **required** [XmlChild] of this XML element.
    ///
    /// Also see notes on safety in [Self::required_property].
    fn required_child<T: XmlWrapper>(
        &self,
        name: &str,
        namespace_url: &str,
    ) -> RequiredDynamicChild<T> {
        RequiredDynamicChild::new(self.xml_element(), name, namespace_url)
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
    /// Note that this also applies to the "default" empty namespace: if the empty namespace is
    /// used, the method will add `xmlns=""` to the detached tag in order to reset
    /// the default namespace in the root of the detached sub-tree to the empty namespace.
    ///
    /// ### Errors
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

    /// Returns `true` if this element is in the detached state (i.e. it has no parent).
    fn is_detached(&self) -> bool {
        let element = self.raw_element();
        let doc = self.read_doc();
        element.parent(doc.deref()).is_none()
    }

    /// Try to attach this [XmlWrapper] into the given `parent` [XmlWrapper] as a new child at
    /// the given `position`. If `position` is not given, the child is inserted as the
    /// last element. The method adjusts namespace declarations to ensure the sub-tree is valid
    /// in its new context.
    ///
    /// The method takes all namespaces which are declared directly on `self`, and tries
    /// to propagate them to the root element reachable from `parent` (i.e. either a document
    /// root, or a root of a detached sub-tree where `parent` resides). Specifically:
    ///  - The declaration of a default namespace cannot be propagated. However, it can be removed
    ///    if the root declares the same default namespace.
    ///  - The prefixes which are already declared with matching namespace URLs are removed.
    ///  - The prefixes which are not declared anywhere on the path to root
    ///    can be propagated to root.
    ///  - Other namespaces cannot be propagated, because their prefix is already declared,
    ///    but with another URL. Hence they stay declared only for the newly attached sub-tree.
    ///
    /// Note that if the attached sub-tree uses an empty "default" namespace, then it will have
    /// `xmlns=""` set.
    ///
    /// ### Errors
    ///  - If `self` is not detached or is the container element.
    ///  - If `position > parent.children().len()`.
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
