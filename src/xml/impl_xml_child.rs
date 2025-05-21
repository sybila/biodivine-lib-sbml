use crate::constants::namespaces::Namespace;
use crate::xml::xml_child::{OptionalXmlChild, RequiredXmlChild};
use crate::xml::{XmlChild, XmlElement, XmlWrapper};
use std::marker::PhantomData;

/// [DynamicChild] is an implementation of [XmlChild] that uses a child name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual children, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct DynamicChild<'a, T: XmlWrapper> {
    parent: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

/// [Child] is an implementation of [XmlChild] that uses a tag name that is known
/// at compile time. As such, it is faster than [DynamicChild], but less flexible.
pub struct Child<'a, T: XmlWrapper> {
    parent: &'a XmlElement,
    name: &'static str,
    namespace: Namespace,
    _marker: PhantomData<T>,
}

/// An [OptionalXmlChild] extension of a [DynamicChild].
pub struct OptionalDynamicChild<'a, T: XmlWrapper>(DynamicChild<'a, T>);
/// A [RequiredXmlChild] extension of a [DynamicChild].
pub struct RequiredDynamicChild<'a, T: XmlWrapper>(DynamicChild<'a, T>);

/// An [OptionalXmlChild] extension of a [Child].
pub struct OptionalChild<'a, T: XmlWrapper>(Child<'a, T>);
/// A [RequiredXmlChild] extension of a [Child].
pub struct RequiredChild<'a, T: XmlWrapper>(Child<'a, T>);

impl<T: XmlWrapper> DynamicChild<'_, T> {
    /// Create a new instance of a [DynamicChild] for the given `element` and `name`.
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> DynamicChild<'a, T> {
        DynamicChild {
            parent,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }
}

impl<T: XmlWrapper> Child<'_, T> {
    pub fn new<'a>(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> Child<'a, T> {
        Child {
            parent,
            name,
            namespace,
            _marker: PhantomData,
        }
    }
}

impl<T: XmlWrapper> OptionalDynamicChild<'_, T> {
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> OptionalDynamicChild<'a, T> {
        OptionalDynamicChild(DynamicChild::new(parent, name))
    }
}

impl<T: XmlWrapper> RequiredDynamicChild<'_, T> {
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> RequiredDynamicChild<'a, T> {
        RequiredDynamicChild(DynamicChild::new(parent, name))
    }
}

impl<T: XmlWrapper> OptionalChild<'_, T> {
    pub fn new<'a>(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> OptionalChild<'a, T> {
        OptionalChild(Child::new(parent, name, namespace))
    }
}

impl<T: XmlWrapper> RequiredChild<'_, T> {
    pub fn new<'a>(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> RequiredChild<'a, T> {
        RequiredChild(Child::new(parent, name, namespace))
    }
}

impl<T: XmlWrapper> XmlChild<T> for DynamicChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.parent
    }

    fn simple_name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: XmlWrapper> XmlChild<T> for Child<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.parent
    }

    fn simple_name(&self) -> &str {
        self.name
    }

    fn namespace(&self) -> Option<Namespace> {
        Some(self.namespace)
    }

    fn quantified_name(&self, _write_doc: bool) -> Result<String, String> {
        let prefix: String = {
            let doc = self.parent().read_doc();
            self.parent()
                .element
                .closest_prefix(&doc, self.namespace.1)
                .unwrap_or(self.namespace.0)
                .to_string()
        };
        Ok(format!("{}:{}", prefix, self.name))
    }
}

impl<T: XmlWrapper> XmlChild<T> for OptionalDynamicChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent
    }

    fn simple_name(&self) -> &str {
        self.0.name.as_str()
    }
}

impl<T: XmlWrapper> XmlChild<T> for RequiredDynamicChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent
    }

    fn simple_name(&self) -> &str {
        self.0.name.as_str()
    }
}

impl<T: XmlWrapper> XmlChild<T> for OptionalChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent
    }

    fn simple_name(&self) -> &str {
        self.0.name
    }

    fn namespace(&self) -> Option<Namespace> {
        self.0.namespace()
    }
}

impl<T: XmlWrapper> XmlChild<T> for RequiredChild<'_, T> {
    fn parent(&self) -> &XmlElement {
        self.0.parent
    }

    fn simple_name(&self) -> &str {
        self.0.name
    }

    fn namespace(&self) -> Option<Namespace> {
        self.0.namespace()
    }
}

impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalDynamicChild<'_, T> {}
impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredDynamicChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredChild<'_, T> {}
