//! A thin, type-safe layer over [biodivine_lib_xml_dom] used to represent SBML objects.
//!
//! The SBML object model is a set of thin wrappers around the elements of an underlying XML
//! document. This module provides the abstractions that make such wrappers convenient:
//!
//!  - [XmlElement]: a thin newtype over a [biodivine_lib_xml_dom] element with explicit
//!    accessors (and no implicit dereference).
//!  - [XmlWrapper]: the trait implemented by every SBML object; it provides access to the
//!    underlying [XmlElement], which is the canonical accessor surface for element data.
//!  - [XmlProperty](crate::xml::XmlProperty) (with the [OptionalXmlProperty](crate::xml::OptionalXmlProperty) and [RequiredXmlProperty](crate::xml::RequiredXmlProperty) variants):
//!    type-safe access to a single attribute of an element.
//!  - [XmlChild](crate::xml::XmlChild) (with the [OptionalXmlChild], [RequiredXmlChild](crate::xml::RequiredXmlChild) and [XmlChildDefault](crate::xml::XmlChildDefault)
//!    variants): type-safe access to a single child element.
//!  - [XmlList](crate::xml::XmlList): a typed list of child elements.
//!  - [XmlPropertyType](crate::xml::XmlPropertyType): conversion between an attribute value and a Rust type.
//!  - [XmlSubtype](crate::xml::XmlSubtype) / [XmlSupertype](crate::xml::XmlSupertype): a safe downcast/upcast mechanism between related object
//!    types.
//!
//! The document model itself (parsing, serialization, node editing, namespace handling and
//! whole-document validation) lives in [biodivine_lib_xml_dom] and is not duplicated here.

/// Implementation of the [XmlElement] struct.
mod xml_element;

/// Declaration of the [XmlWrapper] and [XmlDefault] traits.
mod xml_wrapper;

/// Implementation of the [SbmlObject] trait: the declared XML identity of an SBML object type.
mod xml_object;

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

/// Implementation of the SBML-package-aware child abstraction.
mod impl_sbml_child;

/// Declaration of the SBML packages used by a document: which prefix a package is declared with,
/// and how a declaration is introduced when the object model writes into a package the document
/// does not declare yet.
mod impl_sbml_package;

/// Some primitive [XmlPropertyType] implementations, as declared in SBML
/// specification Section 3.1.
mod impl_xml_property_type;

/// Defines [XmlSubtype] and [XmlSupertype] which are helpful for declaring inheritance
/// hierarchies of [XmlWrapper] types.
mod xml_inheritance;

/// Defines SBML properties which adhere to SBML package prefix resolution rules, which are
/// somewhat more specific than normal XML namespaces.
mod impl_sbml_property;

pub use crate::xml::impl_sbml_child::{OptionalSbmlChild, RequiredSbmlChild, SbmlChild};
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
pub use crate::xml::xml_object::{set_xsi_type, SbmlObject};
pub use crate::xml::xml_property::{OptionalXmlProperty, RequiredXmlProperty, XmlProperty};
pub use crate::xml::xml_property_type::XmlPropertyType;
pub use crate::xml::xml_wrapper::XmlDefault;
pub use crate::xml::xml_wrapper::XmlWrapper;

/// A namespace as this crate names it: the conventional prefix a new declaration is written with,
/// and the URL that identifies the namespace itself.
///
/// Names are compared by URL, so the prefix only decides how the crate *writes* a name; a document
/// that binds the URL under another prefix is read the same way. The type is a pair rather than a
/// struct so that a namespace constant is a `const` value (see
/// [`SbmlObject::ELEMENT_NAMESPACE`]).
pub use crate::constants::namespaces::Namespace;

/// The XML document type used by the SBML object model.
///
/// This is a re-export of [biodivine_lib_xml_dom::Document]; a document is a cheap, cloneable,
/// thread-safe handle, so it can be passed around freely (for example to the constructors of
/// new SBML objects).
pub use biodivine_lib_xml_dom::Document;
