//! The checked cast from an SBML object to the view a package adds.

use biodivine_lib_xml_dom::Element;

use crate::extension::{ExtensionError, ExtensionOf};
use crate::xml::{Namespace, SbmlObject, XmlElement, XmlWrapper};
use crate::Sbml;

/// The cast of an SBML object to the view an SBML *package* adds to it.
///
/// This trait is implemented for every SBML object type, so a caller reaches a package's view of a
/// `Model`, a `Species` or any other object with one method call:
///
/// ```
/// use biodivine_lib_sbml::{Sbml, URL_SBML_CORE};
/// use biodivine_lib_sbml::core::Model;
/// use biodivine_lib_sbml::xml::{OptionalXmlChild, RequiredXmlProperty};
/// use biodivine_lib_sbml::extension::{ExtensionCast, ExtensionOf};
///
/// const MY_EXTENSION: (&str, &str) =
///     ("myext", "http://www.sbml.org/sbml/level3/version2/myext/version1");
///
/// #[derive(Debug)]
/// pub struct MyExtensionSpecies { species: biodivine_lib_sbml::core::Species }
///
/// impl ExtensionOf<biodivine_lib_sbml::core::Species> for MyExtensionSpecies {
///     const PACKAGE: (&'static str, &'static str) = MY_EXTENSION;
///     fn from_object(species: biodivine_lib_sbml::core::Species) -> Self {
///         MyExtensionSpecies { species }
///     }
///     fn object(&self) -> &biodivine_lib_sbml::core::Species { &self.species }
/// }
///
/// // A document that does not declare the package: the cast refuses to produce a view.
/// let document = Sbml::read_str(&format!(
///     r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2">
///          <model><listOfSpecies><species id="s" compartment="c"/></listOfSpecies></model>
///        </sbml>"#
/// ))?;
/// let species = document.model().get().unwrap().species().get().unwrap().get(0);
/// let error = species.cast::<MyExtensionSpecies>().unwrap_err();
/// assert!(error.is_not_declared());
/// assert_eq!(error.package(), MY_EXTENSION.1);
///
/// // The editing-side cast declares the package and then produces the view.
/// let view = species.cast_or_activate::<MyExtensionSpecies>()?;
/// assert_eq!(view.object().id().get().as_str(), "s");
/// assert_eq!(document.declared_package_prefix(MY_EXTENSION), Ok("myext".to_string()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Two different casts
///
/// This crate has another notion of a cast: [`XmlSubtype`](crate::xml::XmlSubtype) and
/// [`XmlSupertype`](crate::xml::XmlSupertype) convert between *wrapper types of the same element*
/// (`LineSegment` and `CubicBezier` are both `<curveSegment>` elements), and their question is
/// "does this element have the shape this type describes?". The question of this trait is a
/// property of the **document**, not of the element: "does the document declare the package this
/// view is written against?". An element is a perfectly good `Species` while the document that
/// contains it never declared the package, and reading package data out of it is refused for that
/// reason; an element of a document that declares the package is not refused even when it carries
/// no package data at all (the accessors report the data as absent, which is what an optional
/// property does).
///
/// # Why the check is "declared" and not "the data is present"
///
/// A package is declared once, on the `<sbml>` element, and it is what says that the document *is*
/// a document of that package. An editing call has to be able to start from a document with no
/// package data at all - that is what the package is for - so the check cannot be "the data is
/// present"; it is "the document says it uses the package", and
/// [`cast_or_activate`](ExtensionCast::cast_or_activate) is how a caller that is about to *write*
/// package data satisfies it.
///
/// # The `required` attribute
///
/// "Declared" is exactly what the specification requires ([`Sbml::declared_package_prefix`] uses
/// the same definition): a namespace declaration on the `<sbml>` element whose URI is the package's
/// URL, **and** a `required` attribute of that namespace. A document may carry either half alone,
/// and then it is not a document this crate reads package data out of. The editing-side cast
/// completes the pair: [`Sbml::activate_package`] declares the missing half (and is idempotent, so
/// calling it on a declared package changes nothing).
///
/// The attribute [`cast_or_activate`](ExtensionCast::cast_or_activate) writes is
/// `required="false"`, which is what the specification means by "a document that carries package
/// content without depending on it"; a package whose content *is* required for interpreting the
/// document has to say so itself, by writing `required="true"`.
///
/// # What a package declaration is not
///
/// Two things this crate does not check, both of which are the package's own business:
///
///  * the *version* of the package. A package this crate does not model has one URL, so any
///    document that binds it is read the same way; and
///  * whether the package content is valid - the rules of the package are what
///    [`ExtensionValidation`](crate::extension::ExtensionValidation) is for.
pub trait ExtensionCast: SbmlObject + Clone {
    /// Views this object as the package's view of it, if the document declares the package.
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError::NotInDocument`] if the object is not part of an SBML document (an
    /// element whose document root is not an `<sbml>` element), and
    /// [`ExtensionError::NotDeclared`] if the document does not declare
    /// [`ExtensionOf::PACKAGE`]. Use [`cast_or_activate`](ExtensionCast::cast_or_activate) when the
    /// caller is allowed to declare the package.
    fn cast<E: ExtensionOf<Self>>(&self) -> Result<E, ExtensionError>;

    /// Views this object as the package's view of it, declaring the package when the document does
    /// not declare it yet.
    ///
    /// This is the cast of an editing call: the document learns that it uses the package
    /// ([`Sbml::activate_package`] declares the package namespace on the `<sbml>` element and adds
    /// the `required="false"` attribute), and the view is then produced. A document that already
    /// declares the package is not modified, so the call is idempotent.
    ///
    /// # Errors
    ///
    /// Returns [`ExtensionError::NotInDocument`] for an object that is not part of an SBML
    /// document, and [`ExtensionError::ActivationFailed`] when the declaration could not be
    /// written.
    fn cast_or_activate<E: ExtensionOf<Self>>(&self) -> Result<E, ExtensionError>;
}

impl<T: SbmlObject + Clone> ExtensionCast for T {
    fn cast<E: ExtensionOf<Self>>(&self) -> Result<E, ExtensionError> {
        require_declared(self.xml_element(), E::PACKAGE)?;
        Ok(<E as ExtensionOf<Self>>::from_object(self.clone()))
    }

    fn cast_or_activate<E: ExtensionOf<Self>>(&self) -> Result<E, ExtensionError> {
        let package = E::PACKAGE;
        let element = self.xml_element();
        let Some(sbml_root) = Sbml::try_for_child_element(element) else {
            return Err(ExtensionError::NotInDocument { package: package.1 });
        };

        if !package_is_declared(element, package) {
            // `activate_package` is the public write path of a package declaration, and it is
            // idempotent: it declares the conventional prefix of the package on the `<sbml>`
            // element unless that exact binding is there, and it writes the `required` attribute
            // unless the namespace already has one with the value `"true"`. It therefore completes
            // a *half* declaration (a namespace binding without the attribute, or an attribute
            // without the binding) as well.
            sbml_root.activate_package(package).map_err(|reason| {
                ExtensionError::ActivationFailed {
                    package: package.1,
                    reason,
                }
            })?;
        }

        // The declaration is re-checked rather than assumed: a package namespace that cannot be
        // declared at all (the empty namespace) and a document that refuses the attribute would
        // otherwise hand the caller a view of a package the document does not declare.
        require_declared(element, package)?;
        Ok(<E as ExtensionOf<Self>>::from_object(self.clone()))
    }
}

/// The declaration check of a cast: the object is in an SBML document and that document declares
/// `package`.
///
/// # Errors
///
/// See [`ExtensionCast::cast`].
fn require_declared(element: &XmlElement, package: Namespace) -> Result<(), ExtensionError> {
    if Sbml::try_for_child_element(element).is_none() {
        return Err(ExtensionError::NotInDocument { package: package.1 });
    }
    if package_is_declared(element, package) {
        Ok(())
    } else {
        Err(ExtensionError::NotDeclared { package: package.1 })
    }
}

/// Whether the document that owns `element` declares `package`.
///
/// The definition is the one the specification gives a package declaration, and the one
/// [`Sbml::declared_package_prefix`] reads: a namespace declaration on the `<sbml>` element whose
/// URI is the package's URL, together with a `required` attribute of that namespace.
///
/// The lookup is by **URL**, so a document that binds the package under a prefix other than the one
/// [`ExtensionOf::PACKAGE`] names is declared just the same - a prefix is a spelling, while the URL
/// is what identifies the namespace. A package whose name is one this crate knows (`fbc`, `layout`,
/// `qual`, `comp`) is also declared when the document binds one of its *published* URLs
/// ([`is_package_namespace_of`](crate::is_package_namespace_of)), which is how
/// [`Sbml::declared_package_prefix`] reads it as well.
fn package_is_declared(element: &XmlElement, package: Namespace) -> bool {
    let Some(sbml_root) = Sbml::try_for_child_element(element) else {
        return false;
    };
    let root = sbml_root.xml_element().element();
    let Some(binding) = package_namespace_declaration(root, package) else {
        return false;
    };
    let url = binding.uri();
    root.attributes().keys().any(|name| {
        name.local_name_str() == "required"
            && name
                .namespace()
                .is_some_and(|namespace| namespace.uri() == url)
    })
}

/// The namespace declaration of the `<sbml>` element that binds `package`, if there is one.
fn package_namespace_declaration(
    root: &Element,
    package: Namespace,
) -> Option<biodivine_lib_xml_dom::Namespace> {
    root.namespace_declarations()
        .into_values()
        .flatten()
        .find(|binding| {
            let url = binding.uri();
            url == package.1 || crate::is_package_namespace_of(url, package.0)
        })
}
