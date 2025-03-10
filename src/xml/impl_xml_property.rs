use super::xml_property::OptionalXmlProperty;
use crate::xml::xml_property::RequiredXmlProperty;
use crate::xml::{XmlElement, XmlProperty, XmlPropertyType};
use std::marker::PhantomData;

/// [DynamicProperty] is an implementation of [XmlProperty] that uses an attribute name given
/// at runtime. It is less efficient (and idiomatic) than using a special type for
/// individual properties, but it is useful if the attribute name is dynamic or otherwise
/// not known at compile time.
pub struct DynamicProperty<'a, T: XmlPropertyType> {
    element: &'a XmlElement,
    name: String,
    _marker: PhantomData<T>,
}

/// [Property] is an implementation of [XmlProperty] that uses an attribute name known
/// at compile time. As such, it is faster than [DynamicProperty], but less flexible.
pub struct Property<'a, T: XmlPropertyType> {
    element: &'a XmlElement,
    name: &'static str,
    _marker: PhantomData<T>,
}

/// An extension of [DynamicProperty] that implements [OptionalXmlProperty].
pub struct OptionalDynamicProperty<'a, T: XmlPropertyType>(DynamicProperty<'a, T>);
/// An extension of [DynamicProperty] that implements [RequiredXmlProperty].
pub struct RequiredDynamicProperty<'a, T: XmlPropertyType>(DynamicProperty<'a, T>);

/// An extension of [Property] that implements [OptionalXmlProperty].
pub struct OptionalProperty<'a, T: XmlPropertyType>(Property<'a, T>);
/// An extension of [Property] that implements [RequiredXmlProperty].
pub struct RequiredProperty<'a, T: XmlPropertyType>(Property<'a, T>);

impl<'a, T: XmlPropertyType> DynamicProperty<'a, T> {
    /// Create a new instance of a [DynamicProperty] for the given `element` and `name`.
    pub fn new(element: &'a XmlElement, name: &str) -> DynamicProperty<'a, T> {
        DynamicProperty {
            element,
            name: name.to_string(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: XmlPropertyType> Property<'a, T> {
    pub fn new(element: &'a XmlElement, name: &'static str) -> Property<'a, T> {
        Property {
            element,
            name,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: XmlPropertyType> OptionalProperty<'a, T> {
    pub fn new(element: &'a XmlElement, name: &'static str) -> OptionalProperty<'a, T> {
        OptionalProperty(Property::new(element, name))
    }
}

impl<'a, T: XmlPropertyType> RequiredProperty<'a, T> {
    pub fn new(element: &'a XmlElement, name: &'static str) -> RequiredProperty<'a, T> {
        RequiredProperty(Property::new(element, name))
    }
}

impl<'a, T: XmlPropertyType> OptionalDynamicProperty<'a, T> {
    pub fn new(element: &'a XmlElement, name: &str) -> OptionalDynamicProperty<'a, T> {
        OptionalDynamicProperty(DynamicProperty::new(element, name))
    }
}

impl<'a, T: XmlPropertyType> RequiredDynamicProperty<'a, T> {
    pub fn new(element: &'a XmlElement, name: &str) -> RequiredDynamicProperty<'a, T> {
        RequiredDynamicProperty(DynamicProperty::new(element, name))
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for DynamicProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.element
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for Property<'_, T> {
    fn element(&self) -> &XmlElement {
        self.element
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for OptionalDynamicProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for RequiredDynamicProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for OptionalProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name
    }
}

impl<T: XmlPropertyType> XmlProperty<T> for RequiredProperty<'_, T> {
    fn element(&self) -> &XmlElement {
        self.0.element
    }

    fn name(&self) -> &str {
        self.0.name
    }
}

impl<T: XmlPropertyType> RequiredXmlProperty<T> for RequiredDynamicProperty<'_, T> {}
impl<T: XmlPropertyType> RequiredXmlProperty<T> for RequiredProperty<'_, T> {}
impl<T: XmlPropertyType> OptionalXmlProperty<T> for OptionalDynamicProperty<'_, T> {}
impl<T: XmlPropertyType> OptionalXmlProperty<T> for OptionalProperty<'_, T> {}
