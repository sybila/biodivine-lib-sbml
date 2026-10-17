//! The SBML `units` attribute of a MathML `cn` element.
//!
//! The attribute is unusual, and the two halves of it are asymmetric on purpose:
//!
//!  * it is an **SBML** attribute that appears on a **MathML** element, so it always belongs to the
//!    SBML namespace and has to be written with a prefix (`sbml:units`), because an attribute never
//!    takes the default namespace and the element's default namespace is MathML (prose
//!    Section 3.4.2);
//!  * the validation layer reads it by *local name* (rules 10220, 10221, 10311 and 10313 all match
//!    the attribute by name, ignoring the namespace), so a document that writes it with another
//!    prefix, or without one, is read the same way — and a reader that insisted on the SBML
//!    namespace would disagree with the rules about what the document means.
//!
//! [`SbmlUnits`] therefore reads by local name and writes the prefixed spelling, declaring the
//! prefix on the element when it is not already in scope. That is the same pattern
//! [`set_xsi_type`](crate::set_xsi_type) uses for the `xsi:type` attribute, and it is the one place
//! in this crate that introduces a *prefixed* declaration for an attribute of a MathML element.

use crate::constants::namespaces::URL_SBML_CORE;
use crate::xml::{OptionalXmlProperty, XmlElement, XmlProperty};
use biodivine_lib_xml_dom::{Namespace, QualifiedName};

/// The conventional prefix the crate writes the SBML namespace with inside a MathML formula.
///
/// The specification uses exactly this prefix in its examples (prose Section 3.4.2), and the corpus
/// follows it; a document that binds another prefix to the same URL is still read correctly, because
/// reading matches the local name.
const SBML_PREFIX: &str = "sbml";

/// The SBML `units` attribute of a MathML `cn` element.
///
/// Obtained from [`Cn::units`](crate::math::Cn::units); behaves like any other optional property
/// (`get`, `get_checked`, `set`, `set_some`, `clear`, `is_set`), with two documented differences:
///
///  * reading matches the attribute by local name, so it also finds an unprefixed or differently
///    prefixed `units` (which is what the validation rules do);
///  * writing produces the prefixed spelling `sbml:units` and declares the prefix on the element if
///    the document does not bind it yet. An existing declaration is reused, never rewritten.
pub struct SbmlUnits<'a> {
    element: &'a XmlElement,
}

impl<'a> SbmlUnits<'a> {
    /// The `units` attribute of `element`.
    pub fn new(element: &'a XmlElement) -> Self {
        SbmlUnits { element }
    }

    /// The expanded name of the written attribute, declaring the `sbml` prefix on this element when
    /// the document does not bind it.
    ///
    /// The declaration is written on the element itself rather than on the document root, because
    /// this accessor is the only user of the prefix: an unprefixed MathML element does not need it,
    /// and declaring it document-wide would leave a declaration that most documents never use.
    fn written_name(&self) -> QualifiedName {
        let namespace = Namespace::prefixed(URL_SBML_CORE, SBML_PREFIX)
            .expect("The SBML core namespace is a valid XML namespace.");
        self.element.ensure_namespace(&namespace);
        QualifiedName::with_namespace("units", &namespace)
            .expect("The attribute name 'units' is a valid XML name.")
    }

    /// The attribute exactly as the document writes it, whatever its namespace is.
    fn existing_name(&self) -> Option<QualifiedName> {
        self.element
            .attributes()
            .keys()
            .find(|it| it.local_name_str() == "units")
            .cloned()
    }
}

impl XmlProperty<String> for SbmlUnits<'_> {
    fn element(&self) -> &XmlElement {
        self.element
    }

    fn simple_name(&self) -> &str {
        "units"
    }

    fn qualified_name(&self, write_doc: bool) -> Result<QualifiedName, String> {
        // The `write_doc` flag is what the trait provides for exactly this asymmetry: a read-only
        // caller must not modify the document, so it gets the name written as it stands (and the
        // overridden readers below do not use the name at all), while a write resolves the prefixed
        // spelling and declares it if necessary.
        if write_doc {
            Ok(self.written_name())
        } else {
            Ok(self.existing_name().unwrap_or_else(|| self.written_name()))
        }
    }

    fn is_set(&self) -> bool {
        self.existing_name().is_some()
    }

    fn get_raw(&self) -> Option<String> {
        let name = self.existing_name()?;
        self.element
            .element()
            .attribute(&name)
            .map(|it| it.as_str().to_string())
    }

    fn get_checked(&self) -> Result<Option<String>, String> {
        Ok(self.get_raw())
    }

    fn clear(&self) -> Result<(), String> {
        if let Some(name) = self.existing_name() {
            self.element.element().remove_attribute(&name);
        }
        Ok(())
    }

    fn set_raw(&self, value: String) -> Result<(), String> {
        let name = self.written_name();
        self.element
            .element()
            .set_attribute_checked(name, value)
            .map_err(|it| it.to_string())
    }
}

impl OptionalXmlProperty<String> for SbmlUnits<'_> {}
