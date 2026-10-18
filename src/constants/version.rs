//! Which level and version a document is read as, and how that is decided.
//!
//! Every table of the validation layer - the element names, the allowed and required attributes,
//! the attribute types and the allowed, required and unique children - describes one *version* of
//! one *level* of SBML. The tables of this crate are the Level 3 Version 2 core and package tables
//! (see [`crate::constants::element`]), and every other version is a delta against them. This
//! module is where a document's level and version are resolved, so that every table lookup can ask
//! for the right one.
//!
//! ## The resolution rule
//!
//! A document is read as a modelled level and version when both of these hold:
//!
//!  1. the **root element's own namespace** is a published SBML root namespace
//!     ([`SBML_ROOT_NAMESPACES`](crate::SBML_ROOT_NAMESPACES)), and
//!  2. the level and version that come out of the root are a pair this crate models
//!     ([`LevelVersion::from_pair`]).
//!
//! The pair is read from the `level` and `version` **attributes** when both are present and
//! parseable, and from the namespace otherwise. The attributes win because they are the
//! document's explicit declaration of what it is, and because the specification makes the
//! namespace a redundant encoding of the same fact: rules `20101`, `20102` and `20103` require the
//! namespace and the two attributes to be *consistent*, and a document that breaks that
//! requirement is invalid whichever of the two a reader follows. The SBML test suite supports this
//! reading: a document whose namespace and attributes disagree (for example `<sbml
//! xmlns="http://www.sbml.org/sbml/level2" level="1" version="1">` with Level 1 content) has an
//! expectation file that lists only the root rule, so the reference implementation reads the
//! content as the level the *attribute* names.
//!
//! Three consequences are worth stating, because they are the behavior a caller observes:
//!
//!  * a document whose root namespace is **not** a published SBML root (an absent namespace, a
//!    foreign one, or an unpublished variant such as `.../level2/version6`) resolves **no model**.
//!    Rule `20101` reports it; nothing else in the document is checked, because nothing in it can
//!    be interpreted;
//!  * a document whose declared level/version is not a pair this crate models resolves **no model**
//!    either, and the `unmodelled-level` lint names the pair. This is how a user of an older level
//!    learns that the crate read the document but did not check it, instead of receiving an empty
//!    report;
//!  * a document of a modelled pair resolves its model and is validated against that pair's
//!    tables, including the three version-dependent core checks that already existed (the MathML
//!    subset of rule `10202`, the class list of rule `10301` and the `rateOf` guards), which used
//!    to be reachable only through the test harness's namespace translation.

use crate::constants::namespaces::{is_core_namespace, level_version_of_namespace};
use crate::xml::XmlElement;

/// A `(level, version)` pair of SBML that this crate models.
///
/// The variants are the pairs the crate has tables for. A pair that is not here resolves no model
/// (see the module documentation), so adding a level or a version is adding a variant and its
/// delta rows, never a new code path.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum LevelVersion {
    /// SBML Level 2 Version 1.
    L2V1,
    /// SBML Level 2 Version 2.
    L2V2,
    /// SBML Level 2 Version 3.
    L2V3,
    /// SBML Level 2 Version 4.
    L2V4,
    /// SBML Level 2 Version 5.
    L2V5,
    /// SBML Level 3 Version 1.
    L3V1,
    /// SBML Level 3 Version 2, the version the base tables are written for.
    L3V2,
}

impl LevelVersion {
    /// Every pair this crate models, in ascending order.
    pub(crate) const ALL: &'static [LevelVersion] = &[
        LevelVersion::L2V1,
        LevelVersion::L2V2,
        LevelVersion::L2V3,
        LevelVersion::L2V4,
        LevelVersion::L2V5,
        LevelVersion::L3V1,
        LevelVersion::L3V2,
    ];

    /// The SBML level.
    pub(crate) const fn level(self) -> u32 {
        match self {
            LevelVersion::L2V1
            | LevelVersion::L2V2
            | LevelVersion::L2V3
            | LevelVersion::L2V4
            | LevelVersion::L2V5 => 2,
            LevelVersion::L3V1 | LevelVersion::L3V2 => 3,
        }
    }

    /// The SBML version within the level.
    pub(crate) const fn version(self) -> u32 {
        match self {
            LevelVersion::L2V1 => 1,
            LevelVersion::L2V2 => 2,
            LevelVersion::L2V3 => 3,
            LevelVersion::L2V4 => 4,
            LevelVersion::L2V5 => 5,
            LevelVersion::L3V1 => 1,
            LevelVersion::L3V2 => 2,
        }
    }

    /// The pair for a `level`/`version` pair, or `None` when this crate does not model it.
    pub(crate) const fn from_pair(level: u32, version: u32) -> Option<LevelVersion> {
        match (level, version) {
            (2, 1) => Some(LevelVersion::L2V1),
            (2, 2) => Some(LevelVersion::L2V2),
            (2, 3) => Some(LevelVersion::L2V3),
            (2, 4) => Some(LevelVersion::L2V4),
            (2, 5) => Some(LevelVersion::L2V5),
            (3, 1) => Some(LevelVersion::L3V1),
            (3, 2) => Some(LevelVersion::L3V2),
            _ => None,
        }
    }
}

/// The level and version the given **root element** declares, when the crate models the pair.
///
/// The rule is documented in this module; `None` means "the document is not read as a modelled
/// level and version", which is either because its root namespace is not a published SBML root
/// (rule `20101`) or because the declared pair is not modelled (the `unmodelled-level` lint).
pub(crate) fn root_level_version(root: &XmlElement) -> Option<LevelVersion> {
    let namespace = root.namespace_url();
    if !is_core_namespace(&namespace) {
        return None;
    }

    let declared = root
        .get_attribute("level")
        .zip(root.get_attribute("version"))
        .and_then(|(level, version)| {
            Some((
                level.trim().parse::<u32>().ok()?,
                version.trim().parse::<u32>().ok()?,
            ))
        });
    if let Some((level, version)) = declared {
        return LevelVersion::from_pair(level, version);
    }

    // One of the two attributes is missing or unparsable: the namespace is the fallback, which is
    // what makes such a document still readable (the root rules report the attribute).
    let (namespace_level, namespace_version) = level_version_of_namespace(&namespace);
    namespace_version.and_then(|version| LevelVersion::from_pair(namespace_level, version))
}

/// Whether the element *is* the `<sbml>` container element of its document.
pub(crate) fn root_is_sbml_element(element: &XmlElement) -> bool {
    element.tag_name() == "sbml"
}

/// Whether the root element of the document this element belongs to is a *published* SBML root.
///
/// This is the condition of rule `20101`, and it is deliberately about the root element's own
/// namespace URL rather than about the presence of an `xmlns` declaration: a document that binds
/// the SBML namespace under a prefix and writes `<a:sbml>` is a valid document, while a document
/// whose `<sbml>` element is in no namespace (or in a foreign one) is not.
pub(crate) fn root_is_published_sbml_root(element: &XmlElement) -> bool {
    element
        .document_root()
        .is_some_and(|root| root.tag_name() == "sbml" && is_core_namespace(&root.namespace_url()))
}

/// Whether the element belongs to a document this crate reads as a modelled level and version.
///
/// This is the gate of the object model: the typed wrappers of this crate describe the Level 3
/// core, so an element of a document whose declared level/version the crate does not model is not
/// an element of one of those types, and resolving it would hand a caller a typed view of a
/// document described by the wrong tables. An element that is not part of an SBML document at all
/// (a detached subtree the constructors build, or an element of a document whose root is not
/// `<sbml>`) passes: there is no document to contradict, and the crate creates its elements in
/// documents it writes itself.
pub(crate) fn element_is_in_modelled_document(element: &XmlElement) -> bool {
    match element.document_root() {
        None => true,
        Some(root) if root.tag_name() != "sbml" => true,
        Some(_) => document_level_version(element).is_some(),
    }
}

/// The versions each level of SBML has published.
///
/// Rule `20103` asks whether the declared `version` is consistent with the namespace, and a
/// namespace that does not name a version - Level 1's, which covers both of its versions - can only
/// answer the question by listing them. The table is also what "a pair the crate could model" means
/// when a level is added to [`LevelVersion`].
pub(crate) fn published_versions(level: u32) -> &'static [u32] {
    match level {
        1 => &[1, 2],
        2 => &[1, 2, 3, 4, 5],
        3 => &[1, 2],
        _ => &[],
    }
}

/// The level and version the document of `element` is read as, when the crate models it.
///
/// This is the accessor the validation layer uses: it resolves the root of the element's document
/// and applies [`root_level_version`]. An element that does not belong to a document, or whose
/// document root is not an `<sbml>` element, resolves nothing.
pub(crate) fn document_level_version(element: &XmlElement) -> Option<LevelVersion> {
    let root = element.document_root()?;
    if root.tag_name() != "sbml" {
        return None;
    }
    root_level_version(&root)
}

/// The level and version of the document of `element`, for the table lookups.
///
/// An element with no document root (a detached subtree, which the constructors produce) is
/// checked against the base tables: a detached element is one this crate created, and the crate
/// writes Level 3 Version 2.
pub(crate) fn document_level_version_or_base(element: &XmlElement) -> LevelVersion {
    document_level_version(element).unwrap_or(LevelVersion::L3V2)
}

/// Whether the document of `element` is read as Level 3 Version 1.
///
/// This is the predicate the version-dependent core checks of this crate ask (the MathML subset of
/// rule `10202`, the class list of rule `10301` and the `rateOf` guards). A document that is not
/// read as a modelled pair is not Version 1: the root rules report that condition, and assuming a
/// version would turn one report into a cascade of them.
pub(crate) fn is_level_3_version_1(element: &XmlElement) -> bool {
    document_level_version(element) == Some(LevelVersion::L3V1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::namespaces::{level_version_of_namespace, SBML_ROOT_NAMESPACES};
    use crate::xml::{Document, XmlElement};

    /// The namespace map is a *set*, not a function from URL to one pair: two published namespaces
    /// cover more than one version (Level 1 Version 1 and Version 2 share `.../level1`), and one of
    /// them is the Level 2 Version 1 namespace.
    #[test]
    fn the_namespace_map_is_a_set_that_pins_the_version_where_it_can() {
        assert_eq!(
            level_version_of_namespace("http://www.sbml.org/sbml/level1"),
            (1, None),
            "Level 1 Version 1 and Version 2 share one namespace, so it pins no version"
        );
        assert_eq!(
            level_version_of_namespace("http://www.sbml.org/sbml/level2"),
            (2, Some(1)),
            "the Level 2 Version 1 namespace is the one without a version segment"
        );
        assert_eq!(
            level_version_of_namespace("http://www.sbml.org/sbml/level2/version4"),
            (2, Some(4))
        );
        assert_eq!(
            level_version_of_namespace("http://www.sbml.org/sbml/level3/version1/core"),
            (3, Some(1))
        );
        assert_eq!(
            level_version_of_namespace("http://www.sbml.org/sbml/level3/version2/core"),
            (3, Some(2))
        );
        // Every published root has an entry (a root without one would panic in the resolution).
        for url in SBML_ROOT_NAMESPACES {
            let _ = level_version_of_namespace(url);
        }
    }

    /// Every pair the crate models round-trips through its level and version, which is what makes
    /// the deltas addressable: a delta row names a [`LevelVersion`], and the tables it changes are
    /// the ones of the pair that value denotes.
    #[test]
    fn every_modelled_pair_round_trips() {
        assert!(!LevelVersion::ALL.is_empty());
        for version in LevelVersion::ALL {
            assert_eq!(
                LevelVersion::from_pair(version.level(), version.version()),
                Some(*version)
            );
            assert!((1..=3).contains(&version.level()));
            assert!(version.version() >= 1);
        }
    }

    /// The resolution rule at its source: the declared attributes win, the namespace is the
    /// fallback when one of them is missing, and a pair the crate does not model resolves nothing.
    #[test]
    fn the_declared_pair_wins_and_the_namespace_is_the_fallback() {
        fn root(source: &str) -> XmlElement {
            let document = Document::parse_string(source).expect("the document is well-formed");
            XmlElement::new(document.root().expect("the document has a root"))
        }

        // The attributes win: the Version 2 namespace with `version="1"` is Version 1.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="1"/>"#
            )),
            Some(LevelVersion::L3V1)
        );
        // ... and the other way round.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3" version="2"/>"#
            )),
            Some(LevelVersion::L3V2)
        );
        // The namespace is the fallback when an attribute is missing.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" version="1"/>"#
            )),
            Some(LevelVersion::L3V1)
        );
        // A declared pair the crate does not model resolves nothing, even when the namespace is
        // one of a level it does model.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" level="3" version="3"/>"#
            )),
            None
        );
        // ... and a declared pair of a level the crate *does* model wins over the namespace, which
        // is the attribute-over-namespace half of the rule (the pair (2, 1) is modelled, the
        // namespace says Level 3 Version 2).
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="2" version="1"/>"#
            )),
            Some(LevelVersion::L2V1)
        );
        // A pair the crate does not model resolves nothing.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://www.sbml.org/sbml/level2/version2" level="2" version="6"/>"#
            )),
            None
        );
        // A namespace no specification defines resolves nothing, whatever the attributes say.
        assert_eq!(
            root_level_version(&root(
                r#"<sbml xmlns="http://example.org/x" level="3" version="2"/>"#
            )),
            None
        );
        assert_eq!(
            root_level_version(&root(r#"<sbml level="3" version="2"/>"#)),
            None
        );
    }
}
