//! The error a cast to an extension view produces.

/// The error of [`ExtensionCast`](crate::extension::ExtensionCast).
///
/// A cast can fail for three reasons, and they are kept apart because they mean different things to
/// a caller: the object is not in an SBML document at all (nothing can be done about it), the
/// document does not declare the package (an editor-side cast can fix it), or the declaration could
/// not be introduced (the document rejected the write).
///
/// Every variant names the package URL the failing cast was for, which is the
/// [`PACKAGE`](crate::extension::ExtensionOf::PACKAGE) of the view type the caller asked for.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExtensionError {
    /// The object is not part of an SBML document, so no package can be declared for it.
    ///
    /// Every cast looks for the `<sbml>` root of the element's document, because a package is
    /// declared there (and because that is where the declaration is in scope for the whole
    /// document). An element that is detached, or whose document root is not an `<sbml>` element,
    /// has no such root, and a package cannot be used with it.
    #[error(
        "The element is not part of an SBML document, so the SBML package '{package}' cannot be \
         used with it."
    )]
    NotInDocument {
        /// The namespace URL of the package the cast was for.
        package: &'static str,
    },

    /// The document does not declare the package.
    ///
    /// A package is declared by the `<sbml>` element: a namespace declaration that binds its URL (or
    /// the URL of one of its published versions, for a package this crate knows), together with a
    /// `<prefix>:required` attribute. An element of the package may be present in the document
    /// without that declaration, and this variant is what tells the reader that it is: reading the
    /// package data of such a document is not supported, because the document does not say that it
    /// uses the package.
    ///
    /// An editor-side cast ([`cast_or_activate`](crate::extension::ExtensionCast::cast_or_activate))
    /// does not return this variant for a document it can write to: it introduces the declaration
    /// first.
    #[error(
        "The SBML document does not declare the SBML package '{package}': the `<sbml>` element \
         carries no namespace declaration for its URL (or no `required` attribute for it)."
    )]
    NotDeclared {
        /// The namespace URL of the package the cast was for.
        package: &'static str,
    },

    /// The package could not be declared, because the document rejected the declaration.
    ///
    /// This is the failure of the *write* half of
    /// [`cast_or_activate`](crate::extension::ExtensionCast::cast_or_activate), i.e. of
    /// [`Sbml::activate_package`](crate::Sbml::activate_package). It happens for a package whose
    /// namespace cannot be a namespace declaration at all (the empty namespace) and for a document
    /// that rejects the `required` attribute value; the `reason` is the message of the failed write.
    #[error("The SBML package '{package}' could not be declared in the document: {reason}")]
    ActivationFailed {
        /// The namespace URL of the package the cast was for.
        package: &'static str,
        /// The reason [`Sbml::activate_package`](crate::Sbml::activate_package) gave.
        reason: String,
    },
}

impl ExtensionError {
    /// The URL of the package the failing cast was for.
    pub fn package(&self) -> &'static str {
        match self {
            ExtensionError::NotInDocument { package }
            | ExtensionError::NotDeclared { package }
            | ExtensionError::ActivationFailed { package, .. } => package,
        }
    }

    /// Whether the failure is the missing declaration of the package (as opposed to a document that
    /// is no SBML document, or a declaration that could not be written).
    pub fn is_not_declared(&self) -> bool {
        matches!(self, ExtensionError::NotDeclared { .. })
    }
}
