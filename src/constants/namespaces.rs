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

/// Whether `url` is the core namespace of a **published** SBML level/version.
///
/// The core namespace of a level is the namespace the content of its `<model>` lives in. For
/// Levels 1 and 2 that is the same URL as the document root; for Level 3 it is the `/core` form.
/// Every published one of them is a member of [`SBML_ROOT_NAMESPACES`], so the two notions coincide
/// and the constant stays the single list: teaching the crate a new level's namespace is a data
/// change in one place.
///
/// This is the predicate the core lookups match on. They used to compare against
/// [`URL_SBML_CORE`] alone, which is why a document of any other published level resolved no
/// object model at all.
pub(crate) fn is_core_namespace(url: &str) -> bool {
    SBML_ROOT_NAMESPACES.contains(&url)
}

/// The `(prefix, url)` namespace of a core namespace URL, if the URL is one.
///
/// The result borrows the constant list, so it is `'static` and can be handed to the accessors
/// that take a [`Namespace`]. The prefix is empty: a core element is written in the document's
/// default namespace.
pub(crate) fn core_namespace_of(url: &str) -> Option<Namespace> {
    SBML_ROOT_NAMESPACES
        .iter()
        .find(|it| **it == url)
        .map(|it| ("", *it))
}

/// The core namespace a document uses, or [`NS_SBML_CORE`] when it has none.
///
/// This is what a write operation needs: creating a child of a resolved document creates it in
/// *that document's* core namespace, so editing a Level 3 Version 1 document does not mix two core
/// namespaces in one file. A document whose root declares no published SBML core namespace (one the
/// crate has just parsed, say) resolves to [`NS_SBML_CORE`], which is the namespace the crate
/// writes with.
pub(crate) fn document_core_namespace_for(document: &biodivine_lib_xml_dom::Document) -> Namespace {
    document
        .root()
        .and_then(|root| {
            core_namespace_of(
                &root
                    .namespace()
                    .map(|it| it.uri().to_string())
                    .unwrap_or_default(),
            )
        })
        .unwrap_or(NS_SBML_CORE)
}

/// The namespace of the *package* `namespace` names, as **this document** binds it.
///
/// A package has one accepted URL per published version ([`FBC_NAMESPACE_URLS`]), so a document
/// that uses FBC Version 1 has its FBC attributes and elements in the Version 1 URL while the
/// crate's accessors are written against the Version 2 one. A read has to follow the document - a
/// lookup by expanded name compares namespace *URLs*, so reading the attribute in the URL this
/// crate writes with would find nothing and report a present attribute as missing.
///
/// The resolution is the package analogue of [`document_core_namespace_for`]: it looks at the
/// namespace bindings that are in scope *at `element`* (a package may be declared on the root or,
/// less commonly, on a nested element), keeps the ones whose URL is an accepted URL of that
/// package, and prefers the binding under the package's conventional prefix. The returned namespace
/// carries the conventional prefix, which is what the crate writes with and what a read can ignore.
///
/// `namespace` that names no package the crate publishes a URL set for is returned unchanged, so a
/// caller cannot lose a core, MathML or foreign namespace by routing it through this function.
pub(crate) fn document_package_namespace_for(
    element: &biodivine_lib_xml_dom::Element,
    namespace: Namespace,
) -> Namespace {
    let Some(prefix) = package_of_namespace_url(namespace.1) else {
        return namespace;
    };
    let Some(urls) = package_urls_of(prefix) else {
        return namespace;
    };
    let mut fallback = None;
    for (declared, binding) in element.namespaces_in_scope() {
        let Some(binding) = binding else {
            continue;
        };
        let Some(url) = urls.iter().copied().find(|it| *it == binding.uri()) else {
            continue;
        };
        if declared.as_ref().map(|it| it.as_str()) == Some(prefix) {
            return (prefix, url);
        }
        fallback.get_or_insert((prefix, url));
    }
    fallback.unwrap_or(namespace)
}

/// The level and the (optional) version a published core namespace encodes.
///
/// The version is optional because two published namespaces cover more than one version: Level 1
/// Version 1 and Level 1 Version 2 share `http://www.sbml.org/sbml/level1`, and Level 2 Version 1
/// is `http://www.sbml.org/sbml/level2` (while `.../level2/version2` and up name their version).
/// A namespace that pins its version is a *set with one member*, which is what makes the map a set
/// rather than a function from URL to a single (level, version) pair.
///
/// # Panics
///
/// Panics for a URL that is not a published core namespace. Callers test
/// [`is_core_namespace`] first.
pub(crate) fn level_version_of_namespace(url: &str) -> (u32, Option<u32>) {
    if url == "http://www.sbml.org/sbml/level1" {
        return (1, None);
    }
    if url == "http://www.sbml.org/sbml/level2" {
        return (2, Some(1));
    }
    let rest = url
        .strip_prefix("http://www.sbml.org/sbml/level")
        .unwrap_or_else(|| panic!("Not a published SBML core namespace: {url}"));
    let mut segments = rest.split('/');
    let level = segments
        .next()
        .and_then(|it| it.parse().ok())
        .unwrap_or_else(|| panic!("Not a published SBML core namespace: {url}"));
    let version = segments
        .next()
        .and_then(|it| it.strip_prefix("version"))
        .and_then(|it| it.parse().ok())
        .unwrap_or_else(|| panic!("Not a published SBML core namespace: {url}"));
    (level, Some(version))
}

pub const URL_PACKAGE_LAYOUT: &str = "http://www.sbml.org/sbml/level3/version1/layout/version1";

pub const URL_PACKAGE_QUAL: &str = "http://www.sbml.org/sbml/level3/version1/qual/version1";

pub const URL_PACKAGE_FBC: &str = "http://www.sbml.org/sbml/level3/version1/fbc/version2";

/// The accepted namespace URL set of the **FBC** package, one entry per published version.
///
/// A package is identified by the URL of its namespace, and a package *version* has its own URL: a
/// document that uses FBC Version 1 or Version 3 content is as much an FBC document as one that
/// uses Version 2, and the crate therefore has to accept all three URLs wherever it asks "is this
/// the FBC package?".
///
/// Every URL is taken from the released specification document of that version in the SBML
/// specification repository. The parenthesis names the file the URL was read from and the number of
/// occurrences it has there:
///
/// | version | URL | source |
/// | --- | --- | --- |
/// | 1 | `…/level3/version1/fbc/version1` | `sbml-level-3/version-1/fbc/spec/sbml-fbc-version-1-release-1.pdf` (5) |
/// | 2 | `…/level3/version1/fbc/version2` | `sbml-level-3/version-1/fbc/spec/sbml-fbc-version-2-release-1.pdf` (6) |
/// | 3 | `…/level3/version1/fbc/version3` | `sbml-level-3/version-1/fbc/spec/sbml-fbc-version-3-release-1.pdf` (5) |
///
/// The two older versions are also the `ns` of the RelaxNG schemas of the package
/// (`RelaxNG/sbml-fbc/sbml-fbc.rng` and `RelaxNG/sbml-fbc-v2/sbml-fbc-v2.rng`, both on line 13).
///
/// Of the SBML syntactic test corpus, 65 documents bind Version 1, 120 bind Version 2 and 78 bind
/// Version 3 (measured over the two Level 3 halves the harness compares, a document counted once).
/// [`URL_PACKAGE_FBC`] is the version this crate *writes* with, i.e. the entry of
/// [`NS_FBC`].
pub const FBC_NAMESPACE_URLS: &[&str] = &[
    "http://www.sbml.org/sbml/level3/version1/fbc/version1",
    URL_PACKAGE_FBC,
    "http://www.sbml.org/sbml/level3/version1/fbc/version3",
];

/// The accepted namespace URL set of the **Layout** package.
///
/// Layout Version 1 is the only published version; its URL is the `ns` of every rule of the
/// package's validation appendix (`sbml-level-3/version-1/layout/apdx-validation.tex`) and of the
/// released specification document
/// (`sbml-level-3/version-1/layout/sbml-layout-version-1-release-1.pdf`, 16 occurrences). 247
/// documents of the compared corpus halves bind it.
pub const LAYOUT_NAMESPACE_URLS: &[&str] = &[URL_PACKAGE_LAYOUT];

/// The accepted namespace URL set of the **Qualitative Models** package.
///
/// Qual Version 1 is the only published version; its URL is the `ns` of every rule of the
/// package's validation appendix (`sbml-level-3/version-1/qual/spec/apdx-validation.tex`) and of its
/// released specification document (`sbml-level-3/version-1/qual/spec/qual-spec.pdf`, 7
/// occurrences). 91 documents of the compared corpus halves bind it.
pub const QUAL_NAMESPACE_URLS: &[&str] = &[URL_PACKAGE_QUAL];

/// The accepted namespace URL set of **Whole Model Hierarchical Composition** (`comp`), one entry
/// per published version.
///
/// The crate does not model the content of this package yet, so these URLs are published as part of
/// the extension surface and for the unit that implements it:
///
/// | version | URL | source |
/// | --- | --- | --- |
/// | 1 | `…/level3/version1/comp/version1` | `sbml-level-3/version-1/comp/spec/sbml-l3v1-comp-v1r3.pdf` (9) |
/// | 2 | `…/level3/version2/comp/version2` | `sbml-level-3/version-2/comp/spec/sbml-l3v2-comp-v2.pdf` (9) |
///
/// The Version 2 URL is the one place where the naming convention misleads: the package version is
/// `2` and the core level/version is `level3/version2`, so the URL ends in `comp/version2`, not in
/// `comp/version1` (the released PDF states it, which is what settles it). 246 documents of the
/// compared corpus halves bind Version 1; **no document binds Version 2**, so that entry is pinned
/// by its own test rather than by the corpus.
pub const COMP_NAMESPACE_URLS: &[&str] = &[
    "http://www.sbml.org/sbml/level3/version1/comp/version1",
    "http://www.sbml.org/sbml/level3/version2/comp/version2",
];

/// The accepted namespace URLs of the package whose conventional prefix is `prefix`.
///
/// The lookup is the *version-aware* one, and it is deliberately separate from the crate-internal
/// `namespace_of_prefix`: that function answers "which namespace does this prefix denote when the
/// crate writes a name", which has exactly one answer per prefix, while this one answers "which
/// URLs does this package accept when the crate reads a document", which has one answer per
/// published version.
///
/// Returns `None` for a prefix the crate does not publish a URL set for. The callers then treat the
/// single URL of the namespace they were given as the whole set, which is what they did before the
/// sets existed.
pub fn package_urls_of(prefix: &str) -> Option<&'static [&'static str]> {
    match prefix {
        "fbc" => Some(FBC_NAMESPACE_URLS),
        "layout" => Some(LAYOUT_NAMESPACE_URLS),
        "qual" => Some(QUAL_NAMESPACE_URLS),
        "comp" => Some(COMP_NAMESPACE_URLS),
        _ => None,
    }
}

/// The package whose accepted URL set contains `url`, as its conventional prefix.
///
/// This is the read-side counterpart of the crate-internal `namespace_of_prefix`: a document that
/// binds the namespace of *any* published version of a package resolves to that package.
pub fn package_of_namespace_url(url: &str) -> Option<&'static str> {
    ["fbc", "layout", "qual", "comp"]
        .into_iter()
        .find(|prefix| is_package_namespace_of(url, prefix))
}

/// Whether `url` is an accepted namespace URL of the package whose conventional prefix is `prefix`.
///
/// A prefix the crate publishes no URL set for accepts only its own namespace URL, so a package
/// that grows a second version is a data change ([`FBC_NAMESPACE_URLS`] and friends), not a code
/// change at every call site.
pub fn is_package_namespace_of(url: &str, prefix: &str) -> bool {
    match package_urls_of(prefix) {
        Some(urls) => urls.contains(&url),
        None => {
            // A package without a published URL set: `namespace_of_prefix` panics for an unknown
            // prefix, so an unknown one accepts nothing.
            KNOWN_NAMESPACES
                .iter()
                .any(|namespace| namespace.0 == prefix && namespace.1 == url)
        }
    }
}

/// The URL of the package `prefix` that the crate writes with, i.e. the URL of its namespace
/// constant ([`NS_FBC`], [`NS_LAYOUT`], [`NS_QUAL`], and [`URL_SBML_CORE`] for the core).
///
/// A write always uses this URL and the conventional prefix; the accepted sets above are for reading
/// a document somebody else wrote. The two questions are separate on purpose: an extension that
/// builds a new element of a package it does not model has to know which URL this crate writes for
/// that package, and it must not confuse it with the set of URLs a document may use.
pub fn written_url_of(prefix: &str) -> Option<&'static str> {
    KNOWN_NAMESPACES
        .iter()
        .find(|namespace| namespace.0 == prefix)
        .map(|namespace| namespace.1)
}

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

/// Whether the document that owns `element` uses the package version the crate models, i.e. binds
/// one of the namespace URLs of that package whose name the tables use.
///
/// A document that declares no version of the package at all counts as using the modelled one: the
/// callers have already established that the package is declared (through
/// [`declared_package_prefix`](crate::Sbml::declared_package_prefix), which accepts every published
/// version), and a package bound under a prefix other than the conventional one is still the
/// package.
pub(crate) fn uses_modelled_package_version(
    xml_element: &crate::xml::XmlElement,
    namespace: crate::xml::Namespace,
) -> bool {
    let Some(prefix) = package_of_namespace_url(namespace.1) else {
        return true;
    };
    let Some(modelled) = written_url_of(prefix) else {
        return true;
    };
    document_package_namespace_for(xml_element.element(), (prefix, modelled)).1 == modelled
}
pub(crate) fn separate_prefix_name(name: &str) -> (&str, &str) {
    match name.split_once(':') {
        Some((prefix, local)) => (prefix, local),
        None => ("", name),
    }
}

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
///
/// The table is keyed by a namespace's *conventional prefix*, one namespace per prefix, and
/// [`namespace_of_prefix`] relies on that: it answers "what does this prefix denote when the crate
/// writes a name", which has exactly one answer. The URLs a package *accepts* on the read path are a
/// different question with one answer per published version, and they live in the URL sets above
/// ([`FBC_NAMESPACE_URLS`], ...), reached through [`package_urls_of`] and
/// [`package_of_namespace_url`]. A package version therefore does not take a second entry here; it
/// takes one in the URL set of its package.
const KNOWN_NAMESPACES: [Namespace; 4] = [NS_SBML_CORE, NS_LAYOUT, NS_QUAL, NS_FBC];

/// The namespace whose conventional prefix is `prefix`.
///
/// The static rule tables name elements and attributes with a `prefix:local` string (for example
/// `fbc:strict`), and the validation code needs the namespace that such a prefix denotes. The
/// lookup is derived from [KNOWN_NAMESPACES], so a namespace and its conventional prefix cannot
/// drift apart.
///
/// The table is keyed by the conventional prefix of a namespace, one namespace per prefix, and
/// `NS_FBC` denotes the `version2` URL of the FBC package - the one this crate *writes*. A
/// document may bind another published version of the same package (see [`FBC_NAMESPACE_URLS`]); the
/// read path resolves that through [`package_urls_of`], not through this function, because one
/// prefix cannot denote two URLs here.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_published_package_version_has_its_url_in_the_set_of_its_package() {
        // One assertion per published version, so a set cannot lose an entry silently. The version
        // numbers are part of the assertion: a URL that does not name the version it is filed under
        // is a typo, not a smaller set.
        assert!(
            FBC_NAMESPACE_URLS.contains(&"http://www.sbml.org/sbml/level3/version1/fbc/version1")
        );
        assert!(FBC_NAMESPACE_URLS.contains(&URL_PACKAGE_FBC));
        assert_eq!(
            FBC_NAMESPACE_URLS
                .iter()
                .filter(|url| url.ends_with("fbc/version3"))
                .count(),
            1
        );
        assert!(LAYOUT_NAMESPACE_URLS.contains(&URL_PACKAGE_LAYOUT));
        assert!(QUAL_NAMESPACE_URLS.contains(&URL_PACKAGE_QUAL));
        assert!(
            COMP_NAMESPACE_URLS.contains(&"http://www.sbml.org/sbml/level3/version1/comp/version1")
        );
        // The Version 2 URL of COMP ends in `comp/version2`: the package version is what the last
        // segment names, not the core version. This entry is the one the corpus cannot pin (no
        // document binds it), so its own assertion is the whole evidence.
        assert!(
            COMP_NAMESPACE_URLS.contains(&"http://www.sbml.org/sbml/level3/version2/comp/version2")
        );
        assert_eq!(COMP_NAMESPACE_URLS.len(), 2);
    }

    #[test]
    fn the_url_sets_are_the_documented_sets_and_nothing_else() {
        // Exact membership, both directions: a set that grew an entry nobody documented would fail
        // here, and so would one that lost an entry.
        assert_eq!(
            FBC_NAMESPACE_URLS,
            &[
                "http://www.sbml.org/sbml/level3/version1/fbc/version1",
                "http://www.sbml.org/sbml/level3/version1/fbc/version2",
                "http://www.sbml.org/sbml/level3/version1/fbc/version3",
            ]
        );
        assert_eq!(LAYOUT_NAMESPACE_URLS, &[URL_PACKAGE_LAYOUT]);
        assert_eq!(QUAL_NAMESPACE_URLS, &[URL_PACKAGE_QUAL]);
        assert_eq!(COMP_NAMESPACE_URLS.len(), 2);
        // Every URL of a set resolves back to the package that owns it, and no URL of another
        // package or of another namespace does.
        for (prefix, urls) in [
            ("fbc", FBC_NAMESPACE_URLS),
            ("layout", LAYOUT_NAMESPACE_URLS),
            ("qual", QUAL_NAMESPACE_URLS),
            ("comp", COMP_NAMESPACE_URLS),
        ] {
            for url in urls {
                assert_eq!(package_of_namespace_url(url), Some(prefix), "{url}");
                assert!(is_package_namespace_of(url, prefix), "{url}");
            }
            assert_eq!(package_urls_of(prefix), Some(urls));
        }
        assert_eq!(package_of_namespace_url(URL_SBML_CORE), None);
        assert_eq!(package_of_namespace_url(URL_MATHML), None);
        // A package the crate models is not a package of *another* package's set.
        assert!(!is_package_namespace_of(URL_PACKAGE_FBC, "layout"));
        assert!(!is_package_namespace_of(URL_PACKAGE_LAYOUT, "fbc"));
    }

    #[test]
    fn the_package_version_a_document_binds_does_not_change_the_namespace_the_crate_writes() {
        // The two questions this module answers are separate: `NS_FBC` is the version the crate
        // writes with, and it is the second entry of the accepted set (a data change would move both
        // together); the read path accepts all three.
        assert_eq!(namespace_of_prefix("fbc"), NS_FBC);
        assert_eq!(written_url_of("fbc"), Some(URL_PACKAGE_FBC));
        assert_eq!(FBC_NAMESPACE_URLS[1], URL_PACKAGE_FBC);
        assert_eq!(written_url_of("layout"), Some(URL_PACKAGE_LAYOUT));
        assert_eq!(written_url_of("qual"), Some(URL_PACKAGE_QUAL));
        // A package without a written representation (the crate models no COMP types) has none.
        assert_eq!(written_url_of("comp"), None);
        assert_eq!(package_urls_of("spatial"), None);
    }

    #[test]
    fn the_document_namespace_of_a_package_follows_the_document_and_keeps_other_ones() {
        // A document that binds FBC Version 1: the read path resolves the package to the URL the
        // document uses, so a lookup by expanded name finds the attribute where the document put it.
        let document = biodivine_lib_xml_dom::Document::parse_string(&format!(
            r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:fbc="{}" fbc:required="false" level="3"
                    version="1"/>"#,
            FBC_NAMESPACE_URLS[0]
        ))
        .expect("The test document is well formed.");
        let root = document.root().expect("The document has a root element.");
        assert_eq!(
            document_package_namespace_for(&root, NS_FBC),
            ("fbc", FBC_NAMESPACE_URLS[0])
        );

        // A document that binds the package under a prefix of its own: the URL is what a read
        // compares, so the resolution still finds it.
        let document = biodivine_lib_xml_dom::Document::parse_string(&format!(
            r#"<sbml xmlns="{URL_SBML_CORE}" xmlns:flux="{}" flux:required="false" level="3"
                    version="1"/>"#,
            FBC_NAMESPACE_URLS[0]
        ))
        .expect("The test document is well formed.");
        let root = document.root().expect("The document has a root element.");
        assert_eq!(
            document_package_namespace_for(&root, NS_FBC),
            ("fbc", FBC_NAMESPACE_URLS[0])
        );

        // A namespace that names no packaged the crate publishes a set for is returned unchanged:
        // the function cannot lose a core, MathML or foreign namespace.
        for namespace in [NS_SBML_CORE, NS_MATHML, NS_XSI, NS_EMPTY] {
            assert_eq!(document_package_namespace_for(&root, namespace), namespace);
        }
    }
}
