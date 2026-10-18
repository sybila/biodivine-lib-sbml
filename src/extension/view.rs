//! The view an SBML package adds to a core SBML object.

use crate::xml::{
    Namespace, OptionalSbmlChild, OptionalSbmlProperty, SbmlObject, XmlPropertyType, XmlWrapper,
};

/// A view of a core SBML object as an SBML *package* sees it.
///
/// A package adds data to the object model: an attribute of the package's namespace on a core
/// element (`<model myext:strictness="0.5">`), an element of the package's namespace as a child of a
/// core element (`<listOfMyParameters>`), or both. This trait is how a package declares that view:
/// it names the package, and it converts between the core object and the view.
///
/// The view is a *thin* wrapper: it holds the core object (which is a handle to a document element,
/// so cloning it is cheap) and reaches the package data through the accessors of the
/// [`xml`](crate::xml) module, or through the two helpers of this trait, which apply the package's
/// namespace for the package:
///
/// ```
/// use biodivine_lib_sbml::core::Model;
/// use biodivine_lib_sbml::xml::{
///     OptionalSbmlChild, OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty, XmlList,
/// };
/// use biodivine_lib_sbml::{ExtensionCast, ExtensionOf, Sbml, URL_SBML_CORE};
///
/// /// The package namespace: the URL has the shape of the namespace of a standard package (see
/// /// [`ExtensionOf::PACKAGE`]).
/// const MY_EXTENSION: (&str, &str) =
///     ("myext", "http://www.sbml.org/sbml/level3/version2/myext/version1");
///
/// /// The package's view of a `Model`.
/// pub struct MyExtensionModel {
///     model: Model,
/// }
///
/// impl ExtensionOf<Model> for MyExtensionModel {
///     const PACKAGE: (&'static str, &'static str) = MY_EXTENSION;
///
///     fn from_object(model: Model) -> Self {
///         MyExtensionModel { model }
///     }
///
///     fn object(&self) -> &Model {
///         &self.model
///     }
/// }
///
/// impl MyExtensionModel {
///     /// The package's own attribute on `<model>`.
///     pub fn strictness(&self) -> OptionalSbmlProperty<f64> {
///         self.property("strictness")
///     }
///
///     /// The package's own list of children.
///     pub fn parameters(
///         &self,
///     ) -> OptionalSbmlChild<'_, XmlList<biodivine_lib_sbml::xml::XmlElement>> {
///         self.child("listOfMyParameters")
///     }
/// }
///
/// let document = Sbml::read_str(&format!(
///     r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2"><model/></sbml>"#
/// ))?;
/// let model = document.model().get().expect("The document has a model.");
///
/// // The document does not declare the package, so reading package data out of it is refused ...
/// assert!(model.cast::<MyExtensionModel>().is_err());
///
/// // ... while the editing-side cast declares the package first and then succeeds.
/// let view = model.cast_or_activate::<MyExtensionModel>()?;
/// view.strictness().set_some(&0.5);
/// assert_eq!(view.strictness().get(), Some(0.5));
/// // The declaration the cast introduced is visible in the document itself:
/// assert!(document.declared_package_prefix(MY_EXTENSION).is_ok());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # The package namespace
///
/// [`PACKAGE`](ExtensionOf::PACKAGE) is the `(prefix, url)` pair of the package, exactly as
/// [`SbmlObject::ELEMENT_NAMESPACE`] is the pair of an element's namespace: names are compared by
/// URL, so the prefix only decides how this crate *writes* a new declaration and name.
///
/// **The URL has to have the shape of a namespace of the standard**, i.e.
/// `http://www.sbml.org/sbml/level<level>/version<version>/<package>/version<package-version>`
/// (for example `http://www.sbml.org/sbml/level3/version2/myext/version1`). Rule 10102 of the SBML
/// specification forbids an element or an attribute "in an SBML namespace" that no specification
/// defines, and this crate implements the rule over the URI: a namespace is one of the standard when
/// it is a published root (`SBML_ROOT_NAMESPACES`), a package namespace this crate models, or a URL
/// of the shape above; the only other accepted namespaces are the foreign ones a document payload
/// uses (`xsi`, XHTML, MathML and RDF).
///
/// An extension whose URL is none of those - `http://example.org/myext`, say - is an **undefined
/// SBML namespace**, and then the core type check reports every element and every attribute the
/// extension adds to an SBML document: an element of such a namespace under rule `10102` itself, and
/// an attribute under the rule this crate's tables give the element the attribute is on (a
/// compartment's attributes are `20517`, a species' are `20623`), or under `10102` when the tables
/// have no rule for that element. Those reports are not a mistake of the extension: the
/// specification really does forbid an element or an attribute in a namespace that SBML does not
/// define, so a package that is meant to be used on SBML documents declares a URL of the shape
/// above. The corpus has the case in `syntactic/99107/99107-fail-01-06-10102-sev2-l3v2.xml`, whose
/// `xmlns:appear="http://anything"` makes `<appear:listOfShapes>` a 10102 report.
///
/// Deciding the URL is therefore part of writing the package, and a package that is registered with
/// the SBML project has one of the shape above. `tests/extension_object.rs` of this crate uses
/// `http://example.org/myext` for an element that lives in a *probe* document rather than in an
/// SBML one, which is why no rule sees it; a package that adds content to an SBML document uses a
/// spec-shaped URL - `examples/extension-example.rs` shows where.
pub trait ExtensionOf<Base: SbmlObject + Clone>: Sized {
    /// The namespace of the package, as the `(prefix, url)` pair this crate writes names with.
    ///
    /// See the trait documentation for the shape the URL has to have and for what happens when it
    /// does not (rule `10102` on every element and attribute the package adds).
    const PACKAGE: Namespace;

    /// Wraps `object` into this view.
    ///
    /// This is the unchecked conversion: like the unchecked conversions of the XML layer
    /// (`Model::from(element)`), it promises nothing about the document. A caller reaches a view
    /// through the checked casts of [`ExtensionCast`](crate::extension::ExtensionCast), which is
    /// what decides whether the package is declared.
    fn from_object(object: Base) -> Self;

    /// The core object this view extends.
    fn object(&self) -> &Base;

    /// An optional child of the core object in the namespace of the package.
    ///
    /// The name is the local name of the child (`listOfMyParameters`), and the helper is the
    /// package-aware child abstraction of this crate with the package namespace filled in: it reads
    /// the child in the namespace the *document* binds for the package, and it declares the package
    /// on the `<sbml>` element when a child is written into a document that does not declare it yet.
    /// See [`OptionalSbmlChild`].
    fn child<'a, T: XmlWrapper>(&'a self, name: &'static str) -> OptionalSbmlChild<'a, T>
    where
        Base: 'a,
    {
        OptionalSbmlChild::new(self.object().xml_element(), name, Self::PACKAGE)
    }

    /// An optional attribute of the core object in the namespace of the package.
    ///
    /// The name is the local name of the attribute (`strictness`). The helper fills in both
    /// namespaces the property abstraction needs: the attribute is in the namespace of the package,
    /// while the element it is read from is in the namespace of the base type, which is what makes
    /// the attribute *qualified* - a package attribute of a core element is always written with its
    /// prefix. See [`OptionalSbmlProperty`].
    fn property<'a, T: XmlPropertyType>(&'a self, name: &'static str) -> OptionalSbmlProperty<'a, T>
    where
        Base: 'a,
    {
        OptionalSbmlProperty::new(
            self.object().xml_element(),
            name,
            Self::PACKAGE,
            <Base as SbmlObject>::ELEMENT_NAMESPACE,
        )
    }
}
