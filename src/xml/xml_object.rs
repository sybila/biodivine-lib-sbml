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
use crate::xml::{Document, XmlElement, XmlWrapper};

/// A wrapper type that declares which XML element it stands for.
///
/// ## The contract
///
/// A type implements this trait through the `#[sbml_object(..)]` attribute of its
/// `#[derive(XmlWrapper)]` declaration:
///
/// ```
/// use biodivine_lib_sbml::core::Species;
/// use biodivine_lib_sbml::xml::{Document, SbmlObject, XmlElement, XmlWrapper};
/// use sbml_macros::XmlWrapper;
///
/// // A wrapper of this crate (or of an extension) declares the element it stands for.
/// #[derive(Clone, Debug, XmlWrapper)]
/// #[sbml_object(name = "mySpecies", namespace = <Species as SbmlObject>::ELEMENT_NAMESPACE)]
/// pub struct MySpecies(XmlElement);
///
/// assert_eq!(<MySpecies as SbmlObject>::ELEMENT_NAME, "mySpecies");
/// assert_eq!(<MySpecies as SbmlObject>::element_names(), &["mySpecies"]);
///
/// // The declaration also provides the constructor, so the element a value is created from and
/// // the element the type recognizes are the same by construction.
/// let document = Document::parse_string("<?xml version=\"1.0\"?><probe/>").unwrap();
/// let species = MySpecies::new_empty(document);
/// assert!(MySpecies::matches_element(species.xml_element()));
/// assert!(species.is_element());
/// // ... and the constructor is reachable generically, which is what `CONSTRUCT` is for:
/// let construct = <MySpecies as SbmlObject>::CONSTRUCT.expect("The type declares a constructor.");
/// assert!(construct(Document::parse_string("<?xml version=\"1.0\"?><probe/>").unwrap()).is_element());
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
/// #[sbml_object(name = "myGraph", namespace = ("myext", "http://www.sbml.org/sbml/level3/version2/myext/version1"))]
/// pub struct MyGraph(XmlElement);
///
/// assert_eq!(<MyGraph as SbmlObject>::ELEMENT_NAME, "myGraph");
/// assert_eq!(<MyGraph as SbmlObject>::ELEMENT_NAMESPACE.0, "myext");
/// assert_eq!(
///     <MyGraph as SbmlObject>::ELEMENT_NAMESPACE.1,
///     "http://www.sbml.org/sbml/level3/version2/myext/version1"
/// );
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
/// ### A core element matches the core namespace *family*
///
/// A type whose [`ELEMENT_NAMESPACE`](SbmlObject::ELEMENT_NAMESPACE) is the core namespace matches
/// any **published** core namespace, because the core namespace of a document is the namespace of
/// *its* level and version: the content of a Level 1 or Level 2 document is in the namespace of its
/// root, and Level 3 adds `/core`. The family is the data constant
/// [`SBML_ROOT_NAMESPACES`](crate::SBML_ROOT_NAMESPACES), so teaching the crate a level's namespace
/// is a data change.
///
/// The match also requires that the document this element belongs to be read as a level and version
/// this crate **models** (see `constants::version`). The typed wrappers describe the core of one
/// level and version, so accepting an element of a document described by other tables would hand a
/// caller a typed view that does not describe the document. A document whose level the crate does
/// not model therefore resolves no object model, and `Sbml::validate` names it with the
/// `unmodelled-level` lint instead of reporting an empty result.
///
/// The prefix-agnostic rule is a statement about the *element's* namespace. An attribute never
/// takes a default namespace and this crate reads `xsi:type` through its written prefix (the same
/// path the typed `xsi:type` property of a curve segment uses), so a document that binds the XML
/// Schema instance namespace under a prefix other than `xsi` is not recognized as a curve segment of
/// either variant. That matches how the crate reads the attribute everywhere else: one spelling for
/// "the value of the `xsi:type` attribute", one behaviour.
///
/// ## One identity accepts a set of URLs: the document root
///
/// Every type of this crate stands for elements of exactly one namespace, with one exception:
/// [`Sbml`](crate::Sbml), the `<sbml>` document root, is a valid root in the namespace of *any*
/// published SBML level and version. Its [`matches_element`](SbmlObject::matches_element) therefore
/// compares the element's namespace URL against a public data constant
/// ([`SBML_ROOT_NAMESPACES`](crate::SBML_ROOT_NAMESPACES)) instead of against
/// [`ELEMENT_NAMESPACE`](SbmlObject::ELEMENT_NAMESPACE), which for the root is the namespace this
/// crate *writes* with. The widening is data: a level module that teaches the crate a new root
/// namespace extends the constant, not the type test.
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
    ///
    /// The value is the *written* value of the attribute, not a QName resolved against the
    /// element's namespaces: the SBML documents in the wild write the unprefixed class name (the
    /// SBML syntactic test suite has 605 `xsi:type="CubicBezier"` and 523
    /// `xsi:type="LineSegment"` attributes), and this crate writes and reads that same spelling.
    const XSI_TYPE: Option<&'static str> = None;

    /// The constructor that builds a new, empty element of this type, if the type has one.
    ///
    /// Every wrapper type whose declaration names an element of its own (that is, every type except
    /// the three whose [`SbmlObject::ELEMENT_NAME`] is the name of a concrete sibling:
    /// [`AbstractRule`](crate::core::AbstractRule),
    /// [`AbstractTerm`](crate::qual::AbstractTerm) and
    /// [`Association`](crate::constraint::Association)) gets an inherent
    /// `new_empty(document)` from its `#[sbml_object(..)]` declaration, and this constant is that
    /// same method as a function pointer. It is what makes the identity usable generically: a
    /// caller that only knows the type `T` can build an element of it, and the existence of the
    /// constructor is a property of the declaration rather than of a list maintained elsewhere.
    ///
    /// The function pointer is a *Rust* convenience, not a bindable value: a binding layer (the
    /// PyO3 one included) exposes the inherent `new_empty` of each type instead, because a function
    /// pointer has no direct counterpart in most foreign type systems.
    ///
    /// The element the constructor returns is detached and is not guaranteed to be valid: required
    /// attributes and children are not filled in. See [`SbmlObject::matches_element`] for the check
    /// that the element really is of this type.
    ///
    /// A type whose identity is written by hand rather than by a `#[sbml_object(..)]` declaration
    /// can implement [`SbmlObject`] without having this constructor:
    /// [`Sbml`](crate::Sbml), the document root, is the one such type of this crate, and it declares
    /// no constructor because a new root has to name a level and a version (that is what
    /// `Sbml::default` does).
    const CONSTRUCT: Option<fn(Document) -> Self> = None;

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
        let namespace = element.namespace_url();
        if Self::ELEMENT_NAMESPACE.1 == crate::URL_SBML_CORE {
            // A core element lives in the core namespace of the document's level and version, and
            // every published one of them is a member of `SBML_ROOT_NAMESPACES`: the content of a
            // Level 1 or Level 2 document is in the namespace of its root, and Level 3 adds
            // `/core`. Matching the family rather than one URL is what makes a document of an
            // older level resolve its model at all.
            if !crate::constants::namespaces::is_core_namespace(&namespace) {
                return false;
            }
            if !crate::constants::version::element_is_in_modelled_document(element) {
                // The core namespace of a level the crate does not model yet: the typed view of
                // such a document would be described by the wrong tables, so no type accepts its
                // elements and the document resolves no object model. `Sbml::validate` names the
                // condition with the `unmodelled-level` lint.
                return false;
            }
        } else if namespace != Self::ELEMENT_NAMESPACE.1 {
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

/// Write the `xsi:type` attribute of `element`, declaring the XML Schema instance namespace on the
/// element when it is not in scope yet.
///
/// This is the write half of [`SbmlObject::XSI_TYPE`], and the identity-derived constructor of a
/// type that declares that value calls it: `XmlElement::new_qualified` guarantees the namespace of
/// the *element*, but `xsi:type` belongs to another namespace, whose prefix the new element does
/// not bind yet. Without the declaration the attribute name cannot be resolved, and the element
/// would neither serialize into a document a reader can classify nor satisfy
/// [`SbmlObject::matches_element`] in memory.
///
/// # Panics
///
/// Panics if `value` cannot be written as an XML attribute value. Every value of a
/// `#[sbml_object(..)]` declaration is a plain class name, so this cannot happen for the
/// declarations of this crate.
#[track_caller]
pub fn set_xsi_type(element: &XmlElement, value: &str) {
    let namespace =
        crate::constants::namespaces::to_xml_namespace(crate::constants::namespaces::NS_XSI)
            .expect("The XML Schema instance namespace is a valid XML namespace.");
    element.ensure_namespace(&namespace);
    let name = biodivine_lib_xml_dom::QualifiedName::with_namespace("type", &namespace)
        .expect("`type` is a valid XML name.");
    element
        .element()
        .set_attribute_checked(name, value)
        .expect("`value` is written as an XML attribute value.");
}

#[cfg(test)]
mod tests;
