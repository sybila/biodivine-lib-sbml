use crate::xml::xml_child::{OptionalXmlChild, RequiredXmlChild};
use crate::xml::{XmlChild, XmlElement, XmlWrapper};
use std::marker::PhantomData;

/// [DynamicChild] is an implementation of [XmlChild] that uses a child name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual children, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct DynamicChild<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: String,
    namespace_url: String,
    _marker: PhantomData<T>,
}

/// [Child] is an implementation of [XmlChild] that uses a tag name that is known
/// at compile time. As such, it is faster than [DynamicChild], but less flexible.
pub struct Child<'a, T: XmlWrapper> {
    element: &'a XmlElement,
    name: &'static str,
    namespace_url: &'static str,
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
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &str,
        namespace_url: &str,
    ) -> DynamicChild<'a, T> {
        DynamicChild {
            element,
            name: name.to_string(),
            namespace_url: namespace_url.to_string(),
            _marker: PhantomData,
        }
    }
}

impl<T: XmlWrapper> Child<'_, T> {
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &'static str,
        namespace_url: &'static str,
    ) -> Child<'a, T> {
        Child {
            element,
            name,
            namespace_url,
            _marker: PhantomData,
        }
    }
}

impl<T: XmlWrapper> OptionalDynamicChild<'_, T> {
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &str,
        namespace_url: &str,
    ) -> OptionalDynamicChild<'a, T> {
        OptionalDynamicChild(DynamicChild::new(element, name, namespace_url))
    }
}

impl<T: XmlWrapper> RequiredDynamicChild<'_, T> {
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &str,
        namespace_url: &str,
    ) -> RequiredDynamicChild<'a, T> {
        RequiredDynamicChild(DynamicChild::new(element, name, namespace_url))
    }
}

impl<T: XmlWrapper> OptionalChild<'_, T> {
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &'static str,
        namespace_url: &'static str,
    ) -> OptionalChild<'a, T> {
        OptionalChild(Child::new(element, name, namespace_url))
    }
}

impl<T: XmlWrapper> RequiredChild<'_, T> {
    pub fn new<'a>(
        element: &'a XmlElement,
        name: &'static str,
        namespace_url: &'static str,
    ) -> RequiredChild<'a, T> {
        RequiredChild(Child::new(element, name, namespace_url))
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for DynamicChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn namespace_url(&self) -> &str {
        self.namespace_url.as_str()
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for Child<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.element
    }

    fn name(&self) -> &str {
        self.name
    }

    fn namespace_url(&self) -> &str {
        self.namespace_url
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for OptionalDynamicChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    fn namespace_url(&self) -> &str {
        self.0.namespace_url.as_str()
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for RequiredDynamicChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    fn namespace_url(&self) -> &str {
        self.0.namespace_url.as_str()
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for OptionalChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name
    }

    fn namespace_url(&self) -> &str {
        self.0.namespace_url
    }
}

impl<'a, T: XmlWrapper> XmlChild<T> for RequiredChild<'a, T> {
    fn parent(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name
    }

    fn namespace_url(&self) -> &str {
        self.0.namespace_url
    }
}

impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalDynamicChild<'_, T> {}
impl<T: XmlWrapper> OptionalXmlChild<T> for OptionalChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredDynamicChild<'_, T> {}
impl<T: XmlWrapper> RequiredXmlChild<T> for RequiredChild<'_, T> {}
