use biodivine_xml_doc::Document;
use std::sync::{Arc, RwLock};

/// Implementation of the [XmlElement] struct.
mod xml_element;

/// Declaration of the [XmlWrapper] and [XmlDefault] traits.
mod xml_wrapper;

/// Declaration of the [XmlPropertyType] trait.
mod xml_property_type;

/// Declaration of the [XmlProperty] trait.
mod xml_property;

/// Declaration of the [XmlChild] and [XmlChildDefault] traits.
mod xml_child;

/// Implementation of the [XmlList] struct.
mod xml_list;

/// Implementation of the [Property] and [DynamicProperty] structs,
/// including required/optional variants.
mod impl_xml_property;

/// Implementation of the [Child] and [DynamicChild] structs,
/// including required/optional variants.
mod impl_xml_child;

/// Some primitive [XmlPropertyType] implementations, as declared in SBML
/// specification Section 3.1.
mod impl_xml_property_type;

/// Defines [XmlSubtype] and [XmlSupertype] which are helpful for declaring inheritance
/// hierarchies of [XmlWrapper] types.
mod xml_inheritance;

/// Defines SBML properties which adhere to SBML package prefix resolution rules, which are
/// somewhat more specific than normal XML namespaces.
mod impl_sbml_property;

#[cfg(feature = "python")]
pub mod py;

pub use crate::xml::impl_sbml_property::{
    OptionalSbmlProperty, RequiredSbmlProperty, SbmlProperty,
};
pub use crate::xml::impl_xml_child::{
    Child, DynamicChild, OptionalChild, OptionalDynamicChild, RequiredChild, RequiredDynamicChild,
};
pub use crate::xml::impl_xml_property::{
    DynamicProperty, OptionalDynamicProperty, OptionalProperty, Property, RequiredDynamicProperty,
    RequiredProperty,
};
pub use crate::xml::xml_child::{OptionalXmlChild, RequiredXmlChild, XmlChild, XmlChildDefault};
pub use crate::xml::xml_element::XmlElement;
pub use crate::xml::xml_inheritance::{XmlNamedSubtype, XmlSubtype, XmlSupertype};
pub use crate::xml::xml_list::XmlList;
pub use crate::xml::xml_property::{OptionalXmlProperty, RequiredXmlProperty, XmlProperty};
pub use crate::xml::xml_property_type::XmlPropertyType;
pub use crate::xml::xml_wrapper::XmlDefault;
pub use crate::xml::xml_wrapper::XmlWrapper;

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
pub type XmlDocument = Arc<RwLock<Document>>;
