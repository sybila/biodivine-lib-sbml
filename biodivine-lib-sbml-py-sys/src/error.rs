//! The Python exception hierarchy of the SBML bindings.
//!
//! Rust reports a failed read, write or validation as a typed [`SbmlError`]; Python reports a
//! failure as an exception. The mapping is total and lives in this module alone, so a new Rust
//! variant cannot silently become a generic `Exception`:
//!
//! | Rust | Python |
//! | --- | --- |
//! | [`RustSbmlError::Xml`] | `SbmlXmlError` (the message is the XML layer's own) |
//! | [`RustSbmlError::Cancelled`] | `SbmlCancelledError` |
//! | [`RustSbmlError::NotSbmlDocument`] | `SbmlDocumentError` |
//! | [`RustSbmlError::UnsupportedLevel`] | `SbmlUnsupportedLevelError` |
//! | [`RustSbmlError::Io`] | `SbmlIoError` |
//!
//! An SBML document that *violates a rule* is not an error: it is reported through the issues of
//! [`Sbml::validate`](biodivine_lib_sbml::Sbml::validate). The one exception is a caller that asks
//! for a hard failure (the private `validate_strict` helper), which raises `SbmlValidationError`
//! carrying the list of issues.
//!
//! Every class derives from `SbmlError`, which derives from `Exception`, matching the wrapping of
//! the XML DOM library: an SBML failure is distinguished by its class, not by the base class, and a
//! caller that wants them all catches `SbmlError`.

use biodivine_lib_sbml::SbmlError as RustSbmlError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(
    biodivine_sys.sbml,
    SbmlError,
    PyException,
    "Base class of every error the SBML bindings raise."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlXmlError,
    SbmlError,
    "The document could not be parsed or serialized as XML."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlCancelledError,
    SbmlError,
    "A cancellation trigger fired while the operation was running."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlDocumentError,
    SbmlError,
    "The parsed XML document has no root element."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlUnsupportedLevelError,
    SbmlError,
    "The document declares an SBML level/version this library does not model."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlIoError,
    SbmlError,
    "An underlying I/O operation failed."
);
create_exception!(
    biodivine_sys.sbml,
    SbmlValidationError,
    SbmlError,
    "Strict validation found problems; the list of issues is in `args[1]`."
);

/// Maps a Rust [`SbmlError`] onto the Python exception hierarchy.
pub(crate) fn to_py_err(error: RustSbmlError) -> PyErr {
    let message = error.to_string();
    match error {
        RustSbmlError::Xml(_) => SbmlXmlError::new_err(message),
        // Only the `interrupt` module registers a trigger, and it propagates whatever the signal
        // handler raised instead of this error; the mapping is the closest equivalent nonetheless.
        RustSbmlError::Cancelled(_) => SbmlCancelledError::new_err(message),
        RustSbmlError::NotSbmlDocument => SbmlDocumentError::new_err(message),
        RustSbmlError::UnsupportedLevel { .. } => SbmlUnsupportedLevelError::new_err(message),
        RustSbmlError::Io(_) => SbmlIoError::new_err(message),
        // `SbmlError` is `#[non_exhaustive]`: a variant added later is still an error of this
        // library, so it is reported as the base class rather than as a bare `Exception`.
        _ => SbmlError::new_err(message),
    }
}

/// Turns a `Result<_, String>` conversion failure (an attribute value that does not parse, a
/// malformed identifier) into a `ValueError`.
pub(crate) fn value_err(message: String) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(message)
}

/// Adds the exception classes of this module to `module`.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    module.add("SbmlError", py.get_type::<SbmlError>())?;
    module.add("SbmlXmlError", py.get_type::<SbmlXmlError>())?;
    module.add("SbmlCancelledError", py.get_type::<SbmlCancelledError>())?;
    module.add("SbmlDocumentError", py.get_type::<SbmlDocumentError>())?;
    module.add(
        "SbmlUnsupportedLevelError",
        py.get_type::<SbmlUnsupportedLevelError>(),
    )?;
    module.add("SbmlIoError", py.get_type::<SbmlIoError>())?;
    module.add("SbmlValidationError", py.get_type::<SbmlValidationError>())?;
    Ok(())
}
