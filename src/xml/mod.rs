use std::sync::{Arc, RwLock};
use xml_doc::Document;

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

/// Implementation of the [GenericProperty] struct.
///
/// The module is `pub(crate)` because it is used by some macro generated code.
pub(crate) mod generic_property;

/// Implementation of the [GenericChild] struct.
///
/// The module is `pub(crate)` because it is used by some macro generated code.
pub(crate) mod generic_child;

/// Some primitive [XmlPropertyType] implementations, as declared in SBML
/// specification Section 3.1.
mod impl_property_type;

pub use crate::xml::xml_child::XmlChild;
pub use crate::xml::xml_child::XmlChildDefault;
pub use crate::xml::xml_element::XmlElement;
pub use crate::xml::xml_list::XmlList;
pub use crate::xml::xml_property::XmlProperty;
pub use crate::xml::xml_property_type::XmlPropertyType;
pub use crate::xml::xml_wrapper::XmlDefault;
pub use crate::xml::xml_wrapper::XmlWrapper;

pub use crate::xml::generic_child::GenericChild;
pub use crate::xml::generic_property::GenericProperty;

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
pub type XmlDocument = Arc<RwLock<Document>>;
