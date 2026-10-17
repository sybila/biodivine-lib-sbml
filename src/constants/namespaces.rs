/// A "namespace" is just a pair of strings which specify the (1) default prefix
/// and (2) namespace url.
pub type Namespace = (&'static str, &'static str);

/// The URL of the "core" SBML namespace.
pub const URL_SBML_CORE: &str = "http://www.sbml.org/sbml/level3/version2/core";

/// The root namespace URL of every **published** SBML level/version.
///
/// An SBML document is an `<sbml>` element in one of these namespaces; the *content* of the document
/// is then described by the namespace of its `<model>` and by the level the reader dispatches to.
/// The list is what [`Sbml`](crate::Sbml) uses to recognize a document root, and it is a plain data
/// constant: extending it with the URL of a level the crate learns to read is a data change, not a
/// code change.
///
/// The list is the set of root namespaces defined by a specification document of the
/// [SBML specification repository](http://sbml.org/Documents/Specifications):
///
/// | level/version | URL |
/// | --- | --- |
/// | Level 1 (version 1 and 2 share the namespace) | `http://www.sbml.org/sbml/level1` |
/// | Level 2 version 1 | `http://www.sbml.org/sbml/level2` |
/// | Level 2 version 2 | `http://www.sbml.org/sbml/level2/version2` |
/// | Level 2 version 3 | `http://www.sbml.org/sbml/level2/version3` |
/// | Level 2 version 4 | `http://www.sbml.org/sbml/level2/version4` |
/// | Level 2 version 5 | `http://www.sbml.org/sbml/level2/version5` |
/// | Level 3 version 1 | `http://www.sbml.org/sbml/level3/version1/core` |
/// | Level 3 version 2 (this crate's level) | `http://www.sbml.org/sbml/level3/version2/core` |
///
/// Two namespaces that appear as the root of documents of the SBML syntactic test suite are
/// deliberately *not* here, because no specification defines them: the empty namespace (7 documents
/// whose root declares no namespace at all) and `http://www.sbml.org/sbml/level2/version6`
/// (1 document). The reader stays tolerant — it dispatches on the local name so that it can report
/// issues for a document whose root namespace is wrong — while this constant describes what a
/// *valid* SBML document root looks like.
pub const SBML_ROOT_NAMESPACES: &[&str] = &[
    "http://www.sbml.org/sbml/level1",
    "http://www.sbml.org/sbml/level2",
    "http://www.sbml.org/sbml/level2/version2",
    "http://www.sbml.org/sbml/level2/version3",
    "http://www.sbml.org/sbml/level2/version4",
    "http://www.sbml.org/sbml/level2/version5",
    "http://www.sbml.org/sbml/level3/version1/core",
    URL_SBML_CORE,
];

pub const URL_PACKAGE_LAYOUT: &str = "http://www.sbml.org/sbml/level3/version1/layout/version1";

pub const URL_PACKAGE_QUAL: &str = "http://www.sbml.org/sbml/level3/version1/qual/version1";

pub const URL_PACKAGE_FBC: &str = "http://www.sbml.org/sbml/level3/version1/fbc/version2";
/// The URL of the HTML namespace.
pub const URL_HTML: &str = "http://www.w3.org/1999/xhtml";

/// The URL of the MathML namespace.
pub const URL_MATHML: &str = "http://www.w3.org/1998/Math/MathML";

/// The URL of the XML Schema instance namespace.
///
/// This is the namespace of the `xsi:type` attribute, which discriminates the variants of the
/// layout `curveSegment` element (`LineSegment` and `CubicBezier`) from one another. It is an XML
/// Schema namespace, not an SBML one, so it is written with the conventional `xsi` prefix and never
/// declares an SBML package.
pub const URL_XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";

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

/// The XML Schema instance namespace, whose `xsi:type` attribute discriminates the variants of the
/// layout `curveSegment` element.
pub const NS_XSI: Namespace = ("xsi", URL_XSI);

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

/// Whether a namespace URL is a *well-formed* SBML namespace: the core namespace of a level and
/// version, a package namespace of a Level 3 package (`.../level3/versionN/<package>/versionM`), or
/// one of the published document roots.
///
/// The distinction matters for rule 10102, which is about an element or attribute "in an SBML
/// namespace" that no specification defines: a URL under the SBML tree that is *not* shaped like a
/// namespace of the standard (for example `http://www.sbml.org/sbml/version1`, which the corpus
/// uses) is exactly that case, while a package namespace this crate does not model (spatial, comp,
/// multi, groups, fbc version 3, ...) is a namespace of the standard and its content is the
/// package's business, not an error.
pub(crate) fn is_well_formed_sbml_namespace(url: &str) -> bool {
    if SBML_ROOT_NAMESPACES.contains(&url) || KNOWN_NAMESPACES.iter().any(|it| it.1 == url) {
        return true;
    }
    // `http://www.sbml.org/sbml/level<level>/version<version>/core` for the core namespace, and
    // `http://www.sbml.org/sbml/level<level>/version<version>/<package>/version<version>` for a
    // package namespace. The crate does not model every package, but a URL of this shape is a
    // namespace of the standard all the same, and what it may contain is that package's business.
    let Some(rest) = url.strip_prefix("http://www.sbml.org/sbml/level") else {
        return false;
    };
    let mut segments = rest.split('/');
    let (Some(level), Some(version)) = (segments.next(), segments.next()) else {
        return false;
    };
    if level.is_empty() || !version.starts_with("version") || version.len() == "version".len() {
        return false;
    }
    match segments.next() {
        Some("core") => segments.next().is_none(),
        Some(package) if !package.is_empty() => {
            let Some(package_version) = segments.next() else {
                return false;
            };
            package_version.starts_with("version")
                && package_version.len() > "version".len()
                && segments.next().is_none()
        }
        _ => false,
    }
}

/// Whether a namespace URL belongs to something that is not an SBML namespace at all, and whose
/// content is therefore outside the scope of rule 10102: the XML Schema instance namespace, the
/// XHTML namespace, MathML and RDF (the three namespaces an `annotation` or `notes` payload uses).
pub(crate) fn is_foreign_namespace(url: &str) -> bool {
    [URL_XSI, URL_HTML, URL_MATHML].contains(&url)
        || url.starts_with("http://www.w3.org/1999/02/22-rdf-syntax-ns")
}

/// The namespaces this crate knows about, i.e. the SBML core namespace and the namespaces of the
/// SBML packages with support in this crate.
const KNOWN_NAMESPACES: [Namespace; 4] = [NS_SBML_CORE, NS_LAYOUT, NS_QUAL, NS_FBC];

/// The namespace whose conventional prefix is `prefix`.
///
/// The static rule tables name elements and attributes with a `prefix:local` string (for example
/// `fbc:strict`), and the validation code needs the namespace that such a prefix denotes. The
/// lookup is derived from [KNOWN_NAMESPACES], so a namespace and its conventional prefix cannot
/// drift apart.
///
/// The table is keyed by the conventional prefix of a namespace, one namespace per prefix, so a
/// package version with its own namespace URL takes its own entry next to its constant (`NS_FBC`
/// denotes the `version2` URL of the FBC package).
///
/// # Panics
///
/// Panics for a prefix that is not the conventional prefix of one of the namespaces in
/// [KNOWN_NAMESPACES]. The tables that reach this function use no other prefixes.
pub(crate) fn namespace_of_prefix(prefix: &str) -> Namespace {
    KNOWN_NAMESPACES
        .iter()
        .copied()
        .find(|it| it.0 == prefix)
        .unwrap_or_else(|| panic!("Unknown SBML namespace prefix `{prefix}`."))
}
