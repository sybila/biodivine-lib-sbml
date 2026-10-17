//! Declaration of SBML packages in an SBML document.
//!
//! An SBML package is identified by the URL of its namespace. A document declares the packages it
//! uses on the `<sbml>` element, by binding such a URL to a prefix and adding a `<prefix>:required`
//! attribute that says whether a reader has to understand the package to process the document.
//!
//! This crate writes package names with the *conventional* prefix of the package (the one the
//! package specification recommends: `fbc`, `qual`, `layout`), because the meaning of an element or
//! attribute name is fully determined by its local name and its namespace URL; the prefix that a
//! particular document happens to bind to that URL is irrelevant to every lookup, which compares
//! expanded names. The write path therefore follows a single rule:
//!
//! 1. [`Sbml::activate_package`](crate::Sbml::activate_package) makes sure the conventional prefix of the package is declared on
//!    the `<sbml>` element, which puts it in scope for the whole document, including attributes of
//!    core elements such as `<species fbc:charge="...">`.
//! 2. Every package name is built from the namespace constants of the package, through
//!    `crate::constants::namespaces::qualified_name`, never from a prefix found in the document.
//!
//! Keeping the declaration in one place is what makes the rule checkable: every writer of a package
//! name (the package-aware child and property abstractions, and a first-class view of a package
//! object) goes through it.
//!
//! Reading is namespace-aware rather than prefix-aware, so a document that binds the package URL
//! under a different prefix resolves to the same elements and attributes. Validation of the
//! package declarations themselves is a separate concern and lives in the validation layer.

use crate::constants::namespaces::{qualified_name, to_xml_namespace, Namespace};
use crate::xml::XmlWrapper;
use crate::Sbml;

impl Sbml {
    /// The prefix under which the given SBML package is declared in this document.
    ///
    /// The declaration is looked up among the namespace declarations of the `<sbml>` element
    /// itself, not among the namespace bindings that are merely in scope: a document that declares
    /// a package on a nested element (for example on `<model>`) does not declare it for this
    /// function. The returned prefix is the one the document uses, which can differ from the
    /// conventional prefix of the package; it is reported as it is written, including the empty
    /// prefix of a default namespace declaration.
    ///
    /// The validation layer is this method's consumer: the package-aware checks of species and
    /// reactions compare the returned prefix with the conventional one, and the type check of an
    /// element only asks whether the package is declared at all.
    ///
    /// # Errors
    ///
    /// Returns an error when no declaration on the `<sbml>` element binds the URL of `package`, and
    /// when such a declaration exists but the document does not also carry a `required` attribute
    /// for that URL. The attribute is looked up by its expanded name, so the prefix it uses in the
    /// document does not matter.
    ///
    /// # Example
    ///
    /// ```
    /// use biodivine_lib_sbml::{Sbml, NS_FBC, NS_QUAL, URL_PACKAGE_FBC, URL_SBML_CORE};
    ///
    /// let document = Sbml::read_str(&format!(
    ///     r#"<?xml version="1.0" encoding="UTF-8"?>
    ///        <sbml xmlns="{URL_SBML_CORE}" xmlns:fbc="{URL_PACKAGE_FBC}" fbc:required="false"
    ///              level="3" version="2"/>"#
    /// ))
    /// .expect("The document is well formed.");
    ///
    /// // `fbc` is declared, and under the conventional prefix.
    /// assert_eq!(document.declared_package_prefix(NS_FBC), Ok("fbc".to_string()));
    /// // `qual` is not declared, so it has no prefix in this document.
    /// assert!(document.declared_package_prefix(NS_QUAL).is_err());
    /// ```
    pub fn declared_package_prefix(&self, package: Namespace) -> Result<String, String> {
        let element = self.xml_element().element();
        let Some((prefix, _binding)) = element
            .namespace_declarations()
            .into_iter()
            .find(|(_prefix, binding)| binding.as_ref().is_some_and(|it| it.uri() == package.1))
        else {
            return Err(format!(
                "Namespace for SBML package {} does not exist.",
                package.1
            ));
        };

        let prefix = prefix.map(|it| it.as_str().to_string()).unwrap_or_default();
        let required_name = qualified_name(package, "required")?;
        if element.attribute(&required_name).is_none() {
            return Err(format!(
                "Namespace of SBML package {} does not have a `required` attribute on the `<sbml>` element.",
                package.1
            ));
        }

        Ok(prefix)
    }

    /// Make sure that the given SBML package is declared on the `<sbml>` element of this document.
    ///
    /// The conventional prefix of the package is declared on the `<sbml>` element unless that exact
    /// prefix is already bound to the package URL there, and the `<prefix>:required` attribute is
    /// added with the value `"false"` unless it is already present with the value `"true"`. A
    /// declaration that the library introduces itself is never `required`: a document that requires
    /// a package must say so, but a document that merely carries some package content does not.
    ///
    /// A document that already binds the package URL under a *different* prefix gains the
    /// conventional declaration as well, which is harmless (two prefixes may bind the same URL) and
    /// keeps step 2 of the module rule true. The `required` attribute is keyed by its expanded name,
    /// so an existing `<other>:required` for the same URL is found, and a `"true"` value is never
    /// downgraded to `"false"`; when such an attribute has to be written, it is written with the
    /// conventional prefix, which *replaces* the existing attribute (an attribute map is keyed by
    /// expanded name, so the two spellings cannot coexist).
    ///
    /// No document of the SBML test corpus (and no document under `test-inputs/`) binds a package
    /// URL under a non-conventional prefix, so this last paragraph describes behaviour that the
    /// corpus gate cannot exercise; it is the price of the single naming rule, and it can only make
    /// the document more explicit, never invalid: the declared prefix is exactly the one used by
    /// every name the library writes.
    ///
    /// The declaration is written with the document's own `declare_namespace`, which replaces the
    /// binding that the `<sbml>` element already carries for that prefix.
    ///
    /// The method has nothing to return: a caller builds the name of a written element or attribute
    /// from the namespace constants of the package (`qualified_name(package, local)`), which is
    /// exactly the prefix declared here.
    ///
    /// # Errors
    ///
    /// Returns an error if the namespace of `package` cannot be declared, which is the case for the
    /// empty namespace (every package this crate knows about has a namespace URL), and if the
    /// document rejects the `required` attribute value.
    ///
    /// # Example
    ///
    /// ```
    /// use biodivine_lib_sbml::{Sbml, NS_QUAL, URL_PACKAGE_QUAL, URL_SBML_CORE};
    ///
    /// // A fresh document declares no package at all.
    /// let document = Sbml::read_str(&format!(
    ///     r#"<?xml version="1.0" encoding="UTF-8"?>
    ///        <sbml xmlns="{URL_SBML_CORE}" level="3" version="2"/>"#
    /// ))
    /// .expect("The document is well formed.");
    /// assert!(document.declared_package_prefix(NS_QUAL).is_err());
    ///
    /// // Activating one declares its conventional prefix and a `required="false"` attribute.
    /// document
    ///     .activate_package(NS_QUAL)
    ///     .expect("The qual namespace can be declared.");
    /// assert_eq!(document.declared_package_prefix(NS_QUAL), Ok("qual".to_string()));
    ///
    /// // Activating it again changes nothing: the declaration is already there.
    /// document.activate_package(NS_QUAL).expect("Activating twice is fine.");
    /// assert_eq!(document.declared_package_prefix(NS_QUAL), Ok("qual".to_string()));
    /// ```
    pub fn activate_package(&self, package: Namespace) -> Result<(), String> {
        let element = self.xml_element().element();
        let namespace = to_xml_namespace(package).ok_or_else(|| {
            format!(
                "Namespace of SBML package {} cannot be declared.",
                package.1
            )
        })?;

        let declared_here = element
            .namespace_declarations()
            .get(&namespace.prefix().cloned())
            .and_then(|binding| binding.as_ref())
            .is_some_and(|binding| binding.has_same_uri(&namespace));
        if !declared_here {
            element.declare_namespace(namespace);
        }

        let required_name = qualified_name(package, "required")?;
        if !element
            .attribute(&required_name)
            .is_some_and(|it| it.as_str() == "true")
        {
            element
                .set_attribute_checked(required_name, "false")
                .map_err(|it| it.to_string())?;
        }

        Ok(())
    }
}
