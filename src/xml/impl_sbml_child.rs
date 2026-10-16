use crate::constants::namespaces::{to_xml_namespace, Namespace, NS_SBML_CORE};
use crate::xml::{OptionalXmlChild, RequiredXmlChild, XmlChild, XmlElement, XmlWrapper};
use crate::Sbml;
use biodivine_lib_xml_dom::QualifiedName;
use std::marker::PhantomData;

/// A variant of [crate::xml::Child] that supports SBML packages.
pub struct SbmlChild<'a, T: XmlWrapper> {
    parent: &'a XmlElement,
    name: &'static str,
    element_namespace: Namespace,
    _marker: PhantomData<T>,
}

/// An [OptionalXmlChild] extension of a [SbmlChild].
pub struct OptionalSbmlChild<'a, T: XmlWrapper>(SbmlChild<'a, T>);
/// A [RequiredXmlChild] extension of a [SbmlChild].
pub struct RequiredSbmlChild<'a, T: XmlWrapper>(SbmlChild<'a, T>);

impl<'a, T: XmlWrapper> SbmlChild<'a, T> {
    pub fn new(parent: &'a XmlElement, name: &'static str, namespace: Namespace) -> Self {
        Self {
            parent,
            name,
            element_namespace: namespace,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: XmlWrapper> OptionalSbmlChild<'a, T> {
    pub fn new(parent: &'a XmlElement, name: &'static str, namespace: Namespace) -> Self {
        Self(SbmlChild::new(parent, name, namespace))
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }
}

impl<'a, T: XmlWrapper> RequiredSbmlChild<'a, T> {
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        if self.element_namespace == NS_SBML_CORE {
            // A core element lives in the core namespace, which is resolved without an explicit
            // prefix (against the default SBML namespace declared on the <sbml> element).
            let namespace = to_xml_namespace(self.element_namespace)
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

        let prefix = if write_doc {
            sbml_root.ensure_sbml_package(self.element_namespace, false)?
        } else {
            sbml_root.find_sbml_package(self.element_namespace)?
        };

        let namespace = if prefix.is_empty() {
            biodivine_lib_xml_dom::Namespace::without_prefix(self.element_namespace.1)
        } else {
            biodivine_lib_xml_dom::Namespace::prefixed(self.element_namespace.1, prefix.as_str())
        }
        .map_err(|it| it.to_string())?;

        QualifiedName::with_namespace(self.name, &namespace).map_err(|it| it.to_string())
    }
}
