use crate::constants::namespaces::{qualified_name as make_qualified_name, Namespace};
use crate::xml::xml_child::{OptionalXmlChild, RequiredXmlChild};
use crate::xml::{XmlChild, XmlElement, XmlWrapper};
use biodivine_lib_xml_dom::QualifiedName;
use std::marker::PhantomData;

/// [DynamicChild] is an implementation of [XmlChild] that uses a child name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual children, but it is useful if the tag name is dynamic or otherwise
/// not known at compile time.
///
/// A `DynamicChild` is always in no namespace.
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
    /// Create a new instance of a [DynamicChild] for the given `parent` and `name`.
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> DynamicChild<'a, T> {
        DynamicChild {
            parent,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }
}

impl<T: XmlWrapper> Child<'_, T> {
    /// Create a new instance of a [Child] for the given `parent`, `name` and `namespace`.
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
    /// Create a new instance of an [OptionalDynamicChild] for the given `parent` and `name`.
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> OptionalDynamicChild<'a, T> {
        OptionalDynamicChild(DynamicChild::new(parent, name))
    }
}

impl<T: XmlWrapper> RequiredDynamicChild<'_, T> {
    /// Create a new instance of a [RequiredDynamicChild] for the given `parent` and `name`.
    pub fn new<'a>(parent: &'a XmlElement, name: &str) -> RequiredDynamicChild<'a, T> {
        RequiredDynamicChild(DynamicChild::new(parent, name))
    }
}

impl<T: XmlWrapper> OptionalChild<'_, T> {
    /// Create a new instance of an [OptionalChild] for the given `parent`, `name` and `namespace`.
    pub fn new<'a>(
        parent: &'a XmlElement,
        name: &'static str,
        namespace: Namespace,
    ) -> OptionalChild<'a, T> {
        OptionalChild(Child::new(parent, name, namespace))
    }
}

impl<T: XmlWrapper> RequiredChild<'_, T> {
    /// Create a new instance of a [RequiredChild] for the given `parent`, `name` and `namespace`.
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

    fn qualified_name(&self, _write_doc: bool) -> Result<QualifiedName, String> {
        make_qualified_name(self.namespace, self.name)
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
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

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        self.0.qualified_name(write_doc)
    }
}

impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalDynamicChild<'_, T> {}
impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredDynamicChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredChild<'_, T> {}
