use crate::constants::namespaces::{Namespace, NS_SBML_CORE};
use crate::xml::{OptionalXmlChild, RequiredXmlChild, XmlChild, XmlElement, XmlWrapper};
use crate::Sbml;
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

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        self.0.quantified_name(write_doc)
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

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        self.0.quantified_name(write_doc)
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

    fn quantified_name(&self, write_doc: bool) -> Result<String, String> {
        let Some(sbml_root) = Sbml::try_for_child_element(self.parent) else {
            println!("Root not found.");
            return Err(format!(
                "SBML property `{}` used outside of an SBML document.",
                self.name
            ));
        };

        let prefix = if self.element_namespace == NS_SBML_CORE {
            // Find the core prefix. Core is not declared as a package, hence we can't use
            // ensure/find methods.
            let doc = self.parent().read_doc();
            self.parent()
                .element
                .closest_prefix(&doc, self.element_namespace.1)
                .unwrap_or(self.element_namespace.0)
                .to_string()
        } else if write_doc {
            sbml_root.ensure_sbml_package(self.element_namespace, false)?
        } else {
            sbml_root.find_sbml_package(self.element_namespace)?
        };

        if prefix.is_empty() {
            Ok(self.name.to_string())
        } else {
            Ok(format!("{}:{}", prefix, self.name))
        }
    }
}
