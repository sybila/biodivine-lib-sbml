//! The declared XML identity of an SBML object type.
//!
//! Every SBML object of this crate is a thin wrapper around an element of an XML document, and
//! every wrapper type is a *typed view* of exactly one kind of element. [`SbmlObject`] is that
//! fact, written down: the element names the type recognizes, the namespace they live in, and — for
//! the one type that needs it — the `xsi:type` value that tells it apart from a sibling.
//!
//! The declaration is what makes a *checked* conversion possible. The unchecked conversion of an
//! [`XmlElement`] into a wrapper (`T::from(element)`) is a plain type coercion that promises
//! nothing about the element; [`SbmlObject::try_from_element`] and
//! [`SbmlObject::matches_element`] instead answer the question "is this element a valid `T`?", which
//! is what a caller needs when it has an element and does not know which wrapper describes it (a
//! reader dispatching on a tag name, an editor looking for a specific child, an SBML extension
//! taking over the elements of its package).
//!
//! The identities of the built-in types are declared with the `#[sbml_object(..)]` attribute of the
//! `XmlWrapper` derive, which is why the names cannot drift from the wrappers they describe.

use crate::constants::namespaces::Namespace;
use crate::xml::{XmlElement, XmlWrapper};

/// A wrapper type that declares which XML element it stands for.
///
/// ## The contract
///
/// A type implements this trait through the `#[sbml_object(..)]` attribute of its
/// `#[derive(XmlWrapper)]` declaration:
///
/// ```
/// use biodivine_lib_sbml::core::Species;
/// use biodivine_lib_sbml::xml::{SbmlObject, XmlElement};
/// use sbml_macros::XmlWrapper;
///
/// // A wrapper of this crate (or of an extension) declares the element it stands for.
/// #[derive(Clone, Debug, XmlWrapper)]
/// #[sbml_object(name = "mySpecies", namespace = <Species as SbmlObject>::ELEMENT_NAMESPACE)]
/// pub struct MySpecies(XmlElement);
///
/// assert_eq!(<MySpecies as SbmlObject>::ELEMENT_NAME, "mySpecies");
/// assert_eq!(<MySpecies as SbmlObject>::element_names(), &["mySpecies"]);
/// ```
///
/// The namespace of a type is the pair of its conventional prefix and the URL that identifies the
/// namespace; the URL is what is compared, and the prefix is what a new declaration is written
/// with. An extension declares the namespace of the elements it adds the same way, with a pair of
/// its own:
///
/// ```
/// use biodivine_lib_sbml::xml::{SbmlObject, XmlElement};
/// use sbml_macros::XmlWrapper;
///
/// #[derive(Clone, Debug, XmlWrapper)]
/// #[sbml_object(name = "myGraph", namespace = ("myext", "http://example.org/myext"))]
/// pub struct MyGraph(XmlElement);
///
/// assert_eq!(<MyGraph as SbmlObject>::ELEMENT_NAME, "myGraph");
/// assert_eq!(<MyGraph as SbmlObject>::ELEMENT_NAMESPACE.0, "myext");
/// assert_eq!(<MyGraph as SbmlObject>::ELEMENT_NAMESPACE.1, "http://example.org/myext");
/// ```
///
/// The attribute is a *declaration*, not a validation: it records the fact that this crate's
/// reader produces a `Species` from a `<species>` element of the core namespace, and the checks
/// that police the document are the ones in the validation layer, as before.
///
/// ## What counts as a match
///
/// [`SbmlObject::matches_element`] compares
///
///  * the local name of the element against [`SbmlObject::element_names`], and
///  * the **URI** of the element's namespace against [`SbmlObject::ELEMENT_NAMESPACE`], so a
///    document that binds the same namespace under another prefix (or as the default namespace)
///    matches just as well — this is the same prefix-agnostic rule the reader follows, and
///  * the value of the `xsi:type` attribute against [`SbmlObject::XSI_TYPE`], when the type
///    declares one.
///
/// The prefix-agnostic rule is a statement about the *element's* namespace. An attribute never
/// takes a default namespace and this crate reads `xsi:type` through its written prefix (the same
/// path the typed `xsi:type` property of a curve segment uses), so a document that binds the XML
/// Schema instance namespace under a prefix other than `xsi` is not recognized as a curve segment of
/// either variant. That matches how the crate reads the attribute everywhere else: one spelling for
/// "the value of the `xsi:type` attribute", one behaviour.
///
/// ## A match is not a unique identification
///
/// `matches_element` is a *type test*, not a dispatcher: several types can accept the same
/// element, and usually do. A `<layout:compartmentGlyph>` is a [`CompartmentGlyph`] and also a
/// [`GraphicalObject`], and a `<layout:curveSegment xsi:type="CubicBezier">` is a
/// [`CubicBezier`] and also a [`LineSegment`], because the second of each pair is the declared
/// supertype of the first. A caller that needs the most specific type therefore has to ask for the
/// subtypes it knows (or use [`XmlWrapper`] up- and downcasts), and a
/// caller that treats the answer as unique has to make sure the type it asks about is not a
/// supertype.
///
/// [`CompartmentGlyph`]: crate::layout::CompartmentGlyph
/// [`GraphicalObject`]: crate::layout::GraphicalObject
/// [`CubicBezier`]: crate::layout::CubicBezier
/// [`LineSegment`]: crate::layout::LineSegment
/// [`XmlWrapper`]: crate::xml::XmlWrapper
pub trait SbmlObject: XmlWrapper {
    /// The primary local name of the element this type stands for.
    ///
    /// For a type whose element has several names (a supertype such as
    /// [`GraphicalObject`](crate::layout::GraphicalObject)) this is the first name of
    /// [`SbmlObject::element_names`], which is the authoritative list.
    const ELEMENT_NAME: &'static str;

    /// The namespace the element of this type lives in, as the `(prefix, url)` pair the crate
    /// uses for every SBML namespace.
    ///
    /// The prefix is the conventional one the crate *writes* with; a declaration is matched on
    /// the URI.
    const ELEMENT_NAMESPACE: Namespace;

    /// The value of `xsi:type` that this type is told apart from its siblings by, if it is
    /// discriminated that way.
    ///
    /// Almost every type is discriminated by its local name alone, which is why the default is
    /// `None`. The exception in this crate is [`CubicBezier`](crate::layout::CubicBezier): it and
    /// [`LineSegment`](crate::layout::LineSegment) are both written as
    /// `<layout:curveSegment>`, and only the `xsi:type` attribute says which is which.
    const XSI_TYPE: Option<&'static str> = None;

    /// Every local name this type recognizes.
    ///
    /// The default is the single [`SbmlObject::ELEMENT_NAME`]; a type whose element has several
    /// names overrides this method, and the attribute that declares the type does so automatically
    /// when it is written with `names = [..]` instead of `name = "…"`.
    fn element_names() -> &'static [&'static str] {
        std::slice::from_ref(&Self::ELEMENT_NAME)
    }

    /// Returns `true` if `element` is an element of this type.
    ///
    /// See the documentation of the trait for what is compared and for the reason a match does not
    /// identify a type uniquely.
    fn matches_element(element: &XmlElement) -> bool {
        if element.namespace_url() != Self::ELEMENT_NAMESPACE.1 {
            return false;
        }
        if !Self::element_names().contains(&element.tag_name().as_str()) {
            return false;
        }
        match Self::XSI_TYPE {
            None => true,
            Some(expected) => xsi_type_of(element).as_deref() == Some(expected),
        }
    }

    /// Returns `true` if this object really is an instance of the type it is wrapped in.
    ///
    /// This is the check a caller applies to a value it already has; use
    /// [`SbmlObject::try_from_element`] when the value to test is a raw [`XmlElement`].
    fn is_element(&self) -> bool {
        Self::matches_element(self.xml_element())
    }

    /// Try to view `element` as an object of this type.
    ///
    /// Returns `None` if [`SbmlObject::matches_element`] rejects the element, and the wrapper
    /// otherwise. The wrapper is a view of the element, not a copy, and no part of the document is
    /// modified.
    fn try_from_element(element: XmlElement) -> Option<Self> {
        if Self::matches_element(&element) {
            Some(Self::from(element))
        } else {
            None
        }
    }
}

/// The value of the `xsi:type` attribute of `element`, if the attribute is present and its prefix
/// resolves.
///
/// The name is resolved through the same path the typed property of a curve segment uses
/// (`xsi:type` is an attribute of an XML Schema namespace, not of SBML), so the identity of a type
/// and the property a caller reads agree by construction.
fn xsi_type_of(element: &XmlElement) -> Option<String> {
    let name = crate::xml::xml_property::resolve_attribute_name(element, "xsi:type").ok()?;
    element
        .element()
        .attribute(&name)
        .map(|it| it.as_str().to_string())
}

#[cfg(test)]
mod tests;
