//! Extension of SBML objects and of document validation by an SBML package.
//!
//! SBML Level 3 is modular: a *package* adds elements and attributes of its own namespace on top of
//! the core object model, and specifies validation rules of its own. This module is the surface a
//! package uses when it is written *outside* the crate. It provides two mechanisms and nothing else:
//!
//!  * **The cast** ([`ExtensionOf`] + [`ExtensionCast`]). A package declares a *view* type of a core
//!    object (a `Model`, a `Species`, ...) which holds the object and exposes the package data
//!    through the accessors of the XML layer. [`ExtensionCast::cast`] is the checked conversion from
//!    the object to that view: it succeeds only when the document *declares* the package, so a
//!    caller cannot read package data out of a document that does not have the package in the first
//!    place. [`ExtensionCast::cast_or_activate`] is the editing-side conversion: it declares the
//!    package on the `<sbml>` element (through the public
//!    [`Sbml::activate_package`](crate::Sbml::activate_package)) when the document does not declare
//!    it yet, and then succeeds.
//!  * **Rule injection** ([`ExtensionValidation`]). A package describes its rules in a [`RuleSet`]
//!    and checks them in the same way this crate does - one function per rule, reporting through
//!    [`SbmlIssue::new_in`](crate::SbmlIssue::new_in) - and
//!    [`Sbml::validate_with`](crate::Sbml::validate_with) runs those checks as part of the
//!    document's validation, so a package rule appears in the report exactly as a rule of this
//!    crate does, with its own id, kind and severity.
//!
//! A worked example of a package that adds an element type, a package attribute, an injected rule
//! and a round trip through a file is `examples/extension-example.rs`; it is the reference for this
//! module and it is compiled and run by `tests/extension-example.rs`.
//!
//! ## What a package needs from this crate
//!
//!  * a **namespace**, as the `(prefix, url)` pair this crate writes names with. The URL has to be
//!    shaped like the namespace of a standard package
//!    (`http://www.sbml.org/sbml/level3/version2/<package>/version1`): an element or attribute whose
//!    namespace is neither a namespace of the standard nor a foreign one is reported by rule
//!    `10102` (see [`ExtensionOf::PACKAGE`] for the whole rule);
//!  * the **wrapper declarations** of the elements it adds: `#[derive(XmlWrapper)]` with an
//!    `#[sbml_object(..)]` attribute naming the element and the namespace. The derive generates the
//!    identity (element names, namespace, `xsi:type`) and the `new_empty(document)` constructor, and
//!    it is the same derive this crate uses for its own types, so an extension type is a first-class
//!    SBML object;
//!  * the **accessors** of the objects it extends, built from
//!    [`OptionalSbmlChild`](crate::xml::OptionalSbmlChild),
//!    [`OptionalSbmlProperty`](crate::xml::OptionalSbmlProperty), [`XmlList`](crate::xml::XmlList)
//!    and the other types of the [`xml`](crate::xml) module - or from the helpers of
//!    [`ExtensionOf`], which apply the package namespace for the package;
//!  * optionally, a [`RuleSet`] and the checks that report it ([`ExtensionValidation`]).
//!
//! Everything else a package needs is already public: [`Sbml`](crate::Sbml) is the document handle,
//! [`Sbml::declared_package_prefix`](crate::Sbml::declared_package_prefix) answers which prefix the
//! document uses for a package, and the raw [`XmlElement`](crate::xml::XmlElement) of any object is
//! always reachable, so a package can also work on the document directly.
//!
//! ## A package that is not in this crate
//!
//! A package defined outside the crate is a *separate* module (or crate) that depends on
//! `biodivine-lib-sbml`: the declarations above are public API, the derive macro is exported by
//! `sbml-macros`, and the generated code refers to this crate by its absolute name, so a
//! declaration compiles outside the crate. A package whose author wants *first-class* support (its
//! own module in this crate, its types in `for_each_object_type!`, its rules in the crate's own
//! table) is instead merged into the crate - that is what the `constraint` (FBC), `qual` and
//! `layout` modules are.

mod cast;
mod error;
mod validation;
mod view;

pub use cast::ExtensionCast;
pub use error::ExtensionError;
pub use validation::ExtensionValidation;
pub use view::ExtensionOf;

#[cfg(test)]
mod tests;
