//! An SBML extension declared *outside* the crate.
//!
//! The point of this test is that everything an extension needs is public API: the `XmlWrapper`
//! derive, the `#[sbml_object(..)]` attribute, the identity traits (`SbmlObject`, `XmlWrapper`) and
//! the generated constructor. The declarations here are compiled as a separate crate that depends on
//! `biodivine-lib-sbml` exactly the way a third-party package would, so a `crate::` path that leaks
//! into the generated code fails to compile here instead of failing for an extension author.
//!
//! The extension models a package that adds one element of its own (`<myext:geneProduct>`) and one
//! attribute to a core element, which is the two things the SBML package specification allows a
//! package to do.

use biodivine_lib_sbml::core::{SId, Species};
use biodivine_lib_sbml::xml::{Document, RequiredXmlProperty, SbmlObject, XmlElement, XmlWrapper};
use sbml_macros::{SBase, XmlWrapper};

/// The namespace of the extension, as the declaration writes it: a conventional prefix and the URL
/// that identifies the namespace.
const MY_EXTENSION: (&str, &str) = ("myext", "http://example.org/myext");

/// A wrapper declared by the extension, with its own element name and namespace.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "geneProduct", namespace = MY_EXTENSION)]
pub struct MyGeneProduct(XmlElement);

/// A supertype of the extension: two element names, the first of which belongs to a concrete
/// subtype. Its declaration suppresses the generated constructor, because `new_empty(document)`
/// there would silently build the subtype's element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(
    names = ["geneProductAssociation", "and"],
    namespace = MY_EXTENSION,
    constructor = false
)]
pub struct MyAssociation(XmlElement);

/// The concrete subtype of the declaration above.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "and", namespace = MY_EXTENSION)]
pub struct MyAnd(XmlElement);

/// An element that is discriminated by `xsi:type` rather than by its local name.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "operand", namespace = MY_EXTENSION, xsi_type = "special")]
pub struct MyOperand(XmlElement);

#[test]
fn an_out_of_crate_type_builds_the_element_of_its_declaration() {
    let document = Document::parse_string(r#"<?xml version="1.0"?><probe/>"#)
        .expect("The probe document is well formed.");

    let product = MyGeneProduct::new_empty(document.clone());
    assert_eq!(MyGeneProduct::ELEMENT_NAME, "geneProduct");
    assert_eq!(MyGeneProduct::ELEMENT_NAMESPACE, MY_EXTENSION);
    assert_eq!(product.xml_element().tag_name(), "geneProduct");
    assert_eq!(product.xml_element().namespace_url(), MY_EXTENSION.1);
    assert!(MyGeneProduct::matches_element(product.xml_element()));
    assert!(product.is_element());
    assert!(MyGeneProduct::try_from_element(product.xml_element().clone()).is_some());

    // The identity-based constructor is reachable through the trait as well, which is what a
    // generic caller uses.
    let construct = MyGeneProduct::CONSTRUCT.expect("The type declares a constructor.");
    let product = construct(document.clone());
    assert!(product.is_element());

    // The suppressed constructor is suppressed, and the concrete subtype still has one.
    assert!(MyAssociation::CONSTRUCT.is_none());
    assert!(MyAnd::CONSTRUCT.is_some());
    assert!(MyAnd::new_empty(document.clone()).is_element());

    // The `xsi:type` of an out-of-crate type is written by the generated constructor, together
    // with the namespace declaration that makes the attribute name resolvable.
    let operand = MyOperand::new_empty(document.clone());
    assert_eq!(
        operand.xml_element().get_attribute("type").as_deref(),
        Some("special")
    );
    assert!(MyOperand::matches_element(operand.xml_element()));
}

#[test]
fn an_out_of_crate_type_can_extend_the_behaviour_of_a_core_type() {
    // A package attribute is reachable through the dynamic property API with the extension's own
    // namespace, which is how an extension adds data to a core element without the core library
    // knowing about it.
    let document = Document::parse_string(r#"<?xml version="1.0"?><probe/>"#)
        .expect("The probe document is well formed.");
    let species = Species::new_empty(document);
    let id = SId::try_from("s1").expect("`s1` is a valid SBML identifier.");
    species.id().set(&id);
    assert_eq!(species.id().get().as_str(), "s1");
    assert!(species.is_element());
}
