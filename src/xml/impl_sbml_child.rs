use crate::constants::namespaces::{qualified_name, to_xml_namespace, Namespace, NS_SBML_CORE};
use crate::xml::xml_child::find_child_with_name;
use crate::xml::{OptionalXmlChild, RequiredXmlChild, XmlChild, XmlElement, XmlWrapper};
use crate::Sbml;
use biodivine_lib_xml_dom::QualifiedName;
use std::marker::PhantomData;

/// A variant of [crate::xml::Child] that supports SBML packages.
pub struct SbmlChild<'a, T: XmlWrapper> {
    parent: &'a XmlElement,
    name: &'static str,
    element_namespace: Namespace,
    /// Whether this child belongs to the *object model* of its document, which exists only when
    /// the document resolves a level and version this crate models.
    ///
    /// See [`SbmlChild::new_object_model`].
    object_model: bool,
    _marker: PhantomData<T>,
}

/// An [OptionalXmlChild] extension of a [SbmlChild].
pub struct OptionalSbmlChild<'a, T: XmlWrapper>(SbmlChild<'a, T>);
/// A [RequiredXmlChild] extension of a [SbmlChild].
pub struct RequiredSbmlChild<'a, T: XmlWrapper>(SbmlChild<'a, T>);

impl<'a, T: XmlWrapper> SbmlChild<'a, T> {
    /// Creates a package-aware accessor for the child `name` of `parent`.
    pub fn new(parent: &'a XmlElement, name: &'static str, namespace: Namespace) -> Self {
        Self {
            parent,
            name,
            element_namespace: namespace,
            object_model: false,
            _marker: PhantomData,
        }
    }

    /// A child of the *object model*: one that exists only when the document this element belongs
    /// to resolves a level and version this crate models.
    ///
    /// The typed wrappers of this crate describe the core of one level and version, so handing out
    /// a `<model>` of a document described by other tables would be a typed view that does not
    /// describe the document. The one child constructed this way is [`Sbml::model`], which is the
    /// entry point of the object model: everything else is reached through it, so the condition is
    /// stated once, at the door.
    ///
    /// [`Sbml::model`]: crate::Sbml::model
    pub fn new_object_model(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> Self {
        Self {
            parent,
            name,
            element_namespace: namespace,
            object_model: true,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: XmlWrapper> OptionalSbmlChild<'a, T> {
    /// Creates a package-aware accessor for a child that may be absent.
    pub fn new(parent: &'a XmlElement, name: &'static str, namespace: Namespace) -> Self {
        Self(SbmlChild::new(parent, name, namespace))
    }

    /// An optional child of the object model; see [`SbmlChild::new_object_model`].
    pub fn new_object_model(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> Self {
        Self(SbmlChild::new_object_model(parent, name, namespace))
    }
}

impl<'a, T: XmlWrapper> OptionalXmlChild<T> for OptionalSbmlChild<'a, T> {}

impl<'a, T: XmlWrapper> XmlChild<T> for OptionalSbmlChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent()
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }

    fn namespace(&self) -> Option<Namespace> {
        self.0.namespace()
    }

    fn get_raw(&self) -> Option<XmlElement> {
        self.0.get_raw()
    }

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }
}

impl<'a, T: XmlWrapper> RequiredSbmlChild<'a, T> {
    /// Creates a package-aware accessor for a child the specification requires.
    pub fn new(parent: &'a XmlElement, name: &'static str, namespace: Namespace) -> Self {
        Self(SbmlChild::new(parent, name, namespace))
    }
}

impl<'a, T: XmlWrapper> RequiredXmlChild<T> for RequiredSbmlChild<'a, T> {}

impl<'a, T: XmlWrapper> XmlChild<T> for RequiredSbmlChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent()
    }

    fn simple_name(&self) -> &str {
        self.0.simple_name()
    }

    fn namespace(&self) -> Option<Namespace> {
        self.0.namespace()
    }

    fn get_raw(&self) -> Option<XmlElement> {
        self.0.get_raw()
    }

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for SbmlChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.parent
    }

    fn simple_name(&self) -> &str {
        self.name
    }

    fn namespace(&self) -> Option<Namespace> {
        Some(self.element_namespace)
    }

    fn get_raw(&self) -> Option<XmlElement> {
        if self.object_model
            && !crate::constants::version::element_is_in_modelled_document(self.parent)
        {
            // The document declares a level and version this crate does not model, so it has no
            // object model to hand out: `Sbml::validate` names the condition with the
            // `unmodelled-level` lint.
            return None;
        }
        let name = self.qualified_name(false).ok()?;
        find_child_with_name(self.parent, &name)
    }

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        if self.element_namespace == NS_SBML_CORE {
            // A core element lives in the core namespace, which is resolved without an explicit
            // prefix (against the default SBML namespace declared on the <sbml> element). The core
            // namespace of a document is the namespace of *its* level and version, so the lookup
            // resolves the namespace of the document rather than the one this crate writes with:
            // without that, the children of a Level 3 Version 1 document would not be found (a
            // lookup by expanded name compares namespace URLs, and Version 1's is a different one).
            //
            // Reading needs no declaration, because the lookup compares namespaces, not prefixes.
            let namespace =
                crate::constants::namespaces::document_core_namespace_for(&self.parent.document());
            let namespace = to_xml_namespace(namespace)
                .expect("The SBML core namespace is a valid XML namespace.");
            return QualifiedName::with_namespace(self.name, &namespace)
                .map_err(|it| it.to_string());
        }

        let Some(sbml_root) = Sbml::try_for_child_element(self.parent) else {
            return Err(format!(
                "SBML child `{}` used outside of an SBML document.",
                self.name
            ));
        };

        if write_doc {
            // A written name has to be usable: its prefix must be in scope for the whole document,
            // which is what declaring the package on the `<sbml>` element achieves. Reading needs
            // no declaration, because the lookup below compares namespaces, not prefixes.
            sbml_root.activate_package(self.element_namespace)?;
        }

        // A read follows the *document's* version of the package, exactly as the property layer
        // does and for the same reason (see `impl_sbml_property.rs`); a write uses the crate's own
        // URL.
        let resolved = if write_doc {
            self.element_namespace
        } else {
            crate::constants::namespaces::document_package_namespace_for(
                self.parent.element(),
                self.element_namespace,
            )
        };
        qualified_name(resolved, self.name)
    }
}
