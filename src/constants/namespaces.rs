/// A "namespace" is just a pair of strings which specify the (1) default prefix
/// and (2) namespace url.
type Namespace = (&'static str, &'static str);

/// The URL of the "core" SBML namespace.
pub const URL_SBML_CORE: &str = "http://www.sbml.org/sbml/level3/version2/core";

/// The URL of the HTML namespace.
pub const URL_HTML: &str = "http://www.w3.org/1999/xhtml";

/// The URL of the MathML namespace.
pub const URL_MATHML: &str = "http://www.w3.org/1998/Math/MathML";

/// The URL of the "default" empty namespace.
#[cfg(test)]
pub const URL_EMPTY: &str = "";

/// The "core" SBML namespace. Default prefix for this namespace is empty.
pub const NS_SBML_CORE: Namespace = ("", URL_SBML_CORE);

/// The "core" HTML namespace. Default prefix for this namespace is empty.
#[cfg(test)]
pub const NS_HTML: Namespace = ("", URL_HTML);

/// The MathML namespace. Default prefix for this namespace is empty.
pub const NS_MATHML: (&str, &str) = ("", URL_MATHML);

/// The "default" empty namespace. Default prefix for this namespace is empty.
#[cfg(test)]
pub const NS_EMPTY: (&str, &str) = ("", URL_EMPTY);
