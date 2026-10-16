/// A "namespace" is just a pair of strings which specify the (1) default prefix
/// and (2) namespace url.
pub type Namespace = (&'static str, &'static str);

/// The URL of the "core" SBML namespace.
pub const URL_SBML_CORE: &str = "http://www.sbml.org/sbml/level3/version2/core";

pub const URL_PACKAGE_LAYOUT: &str = "http://www.sbml.org/sbml/level3/version1/layout/version1";

pub const URL_PACKAGE_QUAL: &str = "http://www.sbml.org/sbml/level3/version1/qual/version1";

pub const URL_PACKAGE_FBC: &str = "http://www.sbml.org/sbml/level3/version1/fbc/version2";
/// The URL of the HTML namespace.
pub const URL_HTML: &str = "http://www.w3.org/1999/xhtml";

/// The URL of the MathML namespace.
pub const URL_MATHML: &str = "http://www.w3.org/1998/Math/MathML";

/// The URL of the "default" empty namespace.
pub const URL_EMPTY: &str = "";

/// The "core" SBML namespace. Default prefix for this namespace is empty.
pub const NS_SBML_CORE: Namespace = ("", URL_SBML_CORE);

pub const NS_LAYOUT: Namespace = ("layout", URL_PACKAGE_LAYOUT);

pub const NS_QUAL: Namespace = ("qual", URL_PACKAGE_QUAL);

pub const NS_FBC: Namespace = ("fbc", URL_PACKAGE_FBC);

/// The "core" HTML namespace. Default prefix for this namespace is empty.
pub const NS_HTML: Namespace = ("", URL_HTML);

/// The MathML namespace. Default prefix for this namespace is empty.
pub const NS_MATHML: (&str, &str) = ("", URL_MATHML);

/// The "default" empty namespace. Default prefix for this namespace is empty.
pub const NS_EMPTY: (&str, &str) = ("", URL_EMPTY);

/// Convert a `(prefix, url)` namespace pair into the `xml_dom` representation used by the
/// underlying document.
///
/// The empty namespace (empty URL) has no `xml_dom` counterpart, because `xml_dom` represents
/// "no namespace" as `None`; this function returns `None` for it as well.
///
/// # Panics
///
/// Panics if a non-empty `url`/`prefix` pair is not a valid XML namespace. All constant
/// namespaces used by this crate are valid, so this cannot happen for the values defined here.
pub(crate) fn to_xml_namespace(namespace: Namespace) -> Option<biodivine_lib_xml_dom::Namespace> {
    if namespace.1.is_empty() {
        return None;
    }
    let result = if namespace.0.is_empty() {
        biodivine_lib_xml_dom::Namespace::without_prefix(namespace.1)
    } else {
        biodivine_lib_xml_dom::Namespace::prefixed(namespace.1, namespace.0)
    };
    Some(result.expect("Constant SBML namespaces are valid XML namespaces."))
}

/// Build the expanded name of an element or attribute from a `(prefix, url)` namespace pair
/// and a local `name`.
///
/// A namespace with an empty URL produces a name without a namespace.
pub(crate) fn qualified_name(
    namespace: Namespace,
    name: &str,
) -> Result<biodivine_lib_xml_dom::QualifiedName, String> {
    match to_xml_namespace(namespace) {
        None => biodivine_lib_xml_dom::QualifiedName::without_namespace(name),
        Some(namespace) => biodivine_lib_xml_dom::QualifiedName::with_namespace(name, &namespace),
    }
    .map_err(|it| it.to_string())
}

/// Build an expanded name in the namespace identified by `url`, using the concrete namespace
/// `prefix` that a particular document binds to that URL.
///
/// An empty `prefix` produces a name in the default namespace, which is meaningful for element
/// names but is not how a namespaced attribute can be written.
pub(crate) fn prefixed_qualified_name(
    url: &str,
    prefix: &str,
    name: &str,
) -> Result<biodivine_lib_xml_dom::QualifiedName, String> {
    let namespace = if prefix.is_empty() {
        biodivine_lib_xml_dom::Namespace::without_prefix(url)
    } else {
        biodivine_lib_xml_dom::Namespace::prefixed(url, prefix)
    }
    .map_err(|it| it.to_string())?;
    biodivine_lib_xml_dom::QualifiedName::with_namespace(name, &namespace)
        .map_err(|it| it.to_string())
}
