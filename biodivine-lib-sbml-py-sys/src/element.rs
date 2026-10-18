//! The element handle: xml-dom's `Element`, which is what every SBML object wraps.
//!
//! An SBML object is a typed view of one XML element, and the `element` attribute of every object
//! class is that element. There is no second element class in this crate: the type is
//! [`PyElement`] of `biodivine-lib-xml-dom-py-sys`, so a Python user has one `Element` class across
//! both libraries and can pass elements between them.

pub use biodivine_lib_xml_dom_sys::PyElement;
