//! Tests for the declared object identities.
//!
//! Three kinds of check live here, and they are complementary:
//!
//!  * the **56-row identity table** is exercised row by row — every type must accept its own
//!    element name(s) in its own namespace, and must reject a foreign local name and a foreign
//!    namespace, and a namespace that is the same URI under another prefix must still be accepted;
//!  * the **special cases** get their own tests: the `xsi:type` discrimination of the curve segment
//!    variants (through the crate's own typed property, not through the identity), the thirteen
//!    types with a `default(document)` constructor, and the package types whose element is a core
//!    element (`FbcSpecies`, `FbcReaction`);
//!  * the **source cross-check** reads this crate's own `src/` and fails when the crate *reads* a
//!    type from an element name its declaration does not recognize. This is what keeps the table
//!    honest: the identity is new API that the reader does not consume, so no document-level gate
//!    would notice a wrong alias, and the accessor and creation literals are the only evidence of
//!    which element a wrapper is supposed to describe.

use crate::constants::namespaces::{NS_FBC, NS_HTML, NS_LAYOUT, NS_QUAL, NS_SBML_CORE};
use crate::constraint::{
    And, Association, FbcReaction, FbcSpecies, FluxObjective, GeneProduct, GeneProductAssociation,
    GeneProductRef, Objective, Or,
};
use crate::core::{
    AbstractRule, AlgebraicRule, AssignmentRule, Compartment, Constraint, Delay, Event,
    EventAssignment, FunctionDefinition, InitialAssignment, KineticLaw, LocalParameter, Math,
    Model, ModifierSpeciesReference, Parameter, Priority, RateRule, Reaction, Species,
    SpeciesReference, Trigger, Unit, UnitDefinition,
};
use crate::layout::{
    BoundingBox, CompartmentGlyph, CubicBezier, Curve, Dimensions, GeneralGlyph, GraphicalObject,
    Layout, LineSegment, Point, ReactionGlyph, ReferenceGlyph, SpeciesGlyph, SpeciesReferenceGlyph,
    TextGlyph, XsiType,
};
use crate::qual::{
    AbstractTerm, DefaultTerm, FunctionTerm, QualInput, QualOutput, QualitativeSpecies, Transition,
};
use crate::xml::{
    Document, Namespace, SbmlObject, XmlDefault, XmlElement, XmlProperty, XmlWrapper,
};
use crate::Sbml;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The names of all 56 wrapper types of this crate, in one place, so that the checks below cannot
/// cover 55 of them by accident. Every macro that is applied to this list fails to compile when a
/// type is added to it without an implementation of the trait it exercises.
macro_rules! for_each_object_type {
    ($action:ident) => {
        $action!(
            // core
            Model,
            Compartment,
            Species,
            Parameter,
            UnitDefinition,
            Unit,
            FunctionDefinition,
            InitialAssignment,
            Constraint,
            Reaction,
            SpeciesReference,
            ModifierSpeciesReference,
            KineticLaw,
            LocalParameter,
            Event,
            Trigger,
            Priority,
            Delay,
            EventAssignment,
            AbstractRule,
            AlgebraicRule,
            AssignmentRule,
            RateRule,
            Math,
            // fbc
            FbcSpecies,
            FbcReaction,
            GeneProduct,
            FluxObjective,
            Objective,
            GeneProductAssociation,
            Association,
            GeneProductRef,
            And,
            Or,
            // qual
            QualitativeSpecies,
            Transition,
            QualInput,
            QualOutput,
            AbstractTerm,
            DefaultTerm,
            FunctionTerm,
            // layout
            Layout,
            GraphicalObject,
            CompartmentGlyph,
            SpeciesGlyph,
            ReactionGlyph,
            SpeciesReferenceGlyph,
            GeneralGlyph,
            ReferenceGlyph,
            TextGlyph,
            Curve,
            LineSegment,
            CubicBezier,
            Point,
            Dimensions,
            BoundingBox
        )
    };
}

/// The namespace of the `xsi:type` attribute of a curve segment.
const XSI_NAMESPACE: (&str, &str) = ("xsi", "http://www.w3.org/2001/XMLSchema-instance");

/// An empty document to attach the test elements to.
fn document() -> Document {
    Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8"?><probe/>"#)
        .expect("The probe document is well formed.")
}

/// A detached element with the given local name, namespace and (optional) `xsi:type`.
fn element(name: &str, namespace: Namespace, xsi_type: Option<&str>) -> XmlElement {
    let document = document();
    let element = XmlElement::new_qualified(document, name, namespace);
    if let Some(xsi_type) = xsi_type {
        let xsi = biodivine_lib_xml_dom::Namespace::prefixed(XSI_NAMESPACE.1, XSI_NAMESPACE.0)
            .expect("The xsi namespace is a valid XML namespace.");
        element.element().declare_namespace(xsi.clone());
        let attribute = biodivine_lib_xml_dom::QualifiedName::with_namespace("type", &xsi)
            .expect("`type` is a valid XML name.");
        element.element().set_attribute(attribute, xsi_type);
    }
    element
}

/// One row of the identity table: the declared identity of `$type` is checked against elements
/// that do and do not belong to it.
macro_rules! check_identity_row {
    ($($type:ty),* $(,)?) => {
        $(
            {
                let type_name = stringify!($type);
                let namespace = <$type as SbmlObject>::ELEMENT_NAMESPACE;
                let names = <$type as SbmlObject>::element_names();
                let xsi_type = <$type as SbmlObject>::XSI_TYPE;
                assert!(
                    !names.is_empty(),
                    "{} declares no element name.",
                    type_name
                );
                assert_eq!(
                    <$type as SbmlObject>::ELEMENT_NAME,
                    names[0],
                    "The primary element name of {} must be the first of its recognized names.",
                    type_name
                );
                // The same namespace URL bound under a different conventional prefix.
                let alternative = ("someOtherPrefix", namespace.1);

                for name in names {
                    // The element the type stands for.
                    let own = element(name, namespace, xsi_type);
                    assert!(
                        <$type as SbmlObject>::matches_element(&own),
                        "{} must accept <{}> in its own namespace.",
                        type_name,
                        name
                    );
                    let wrapper = <$type as SbmlObject>::try_from_element(own.clone())
                        .unwrap_or_else(|| panic!("{} must be castable from <{}>.", type_name, name));
                    assert!(wrapper.is_element(), "{} is not an instance of itself.", type_name);
                    assert_eq!(wrapper.xml_element(), &own);

                    // The same element, but in a namespace that is the same URL under a prefix
                    // the crate does not use: prefix-agnostic matching must accept it.
                    let renamed = element(name, alternative, xsi_type);
                    assert!(
                        <$type as SbmlObject>::matches_element(&renamed),
                        "{} must accept <{}> when its namespace URL is bound under another prefix.",
                        type_name,
                        name
                    );

                    // A foreign local name in the right namespace.
                    let foreign_name = element("thisIsNotAnSbmlElementName", namespace, xsi_type);
                    assert!(
                        !<$type as SbmlObject>::matches_element(&foreign_name),
                        "{} must reject a foreign local name.",
                        type_name
                    );

                    // The right local name in a foreign namespace.
                    let foreign_namespace = element(name, NS_HTML, xsi_type);
                    assert!(
                        !<$type as SbmlObject>::matches_element(&foreign_namespace),
                        "{} must reject <{}> in a foreign namespace.",
                        type_name,
                        name
                    );
                    assert!(
                        <$type as SbmlObject>::try_from_element(foreign_namespace.clone()).is_none(),
                        "{} must not be castable from an element of a foreign namespace.",
                        type_name
                    );

                    // A type that is discriminated by `xsi:type` accepts nothing else.
                    if xsi_type.is_some() {
                        let wrong = element(name, namespace, Some("notTheDeclaredType"));
                        assert!(
                            !<$type as SbmlObject>::matches_element(&wrong),
                            "{} must reject an element whose `xsi:type` is another value.",
                            type_name
                        );
                        let missing = element(name, namespace, None);
                        assert!(
                            !<$type as SbmlObject>::matches_element(&missing),
                            "{} must reject an element without `xsi:type`.",
                            type_name
                        );
                    }
                }
            }
        )*
    };
}

#[test]
fn every_declared_identity_accepts_its_own_element_and_rejects_the_others() {
    for_each_object_type!(check_identity_row);
}

/// The identity table itself: a name and a namespace URL per type, for the tests that need to look
/// a type up by name (the source cross-check does).
macro_rules! identity_lookup {
    ($($type:ty),* $(,)?) => {
        vec![
            $(
                (
                    stringify!($type),
                    <$type as SbmlObject>::ELEMENT_NAME,
                    <$type as SbmlObject>::element_names(),
                    <$type as SbmlObject>::ELEMENT_NAMESPACE.1,
                ),
            )*
        ]
    };
}

/// The declared identity of every wrapper type: its name, its primary element name, the element
/// names it recognizes and the URL of its namespace.
fn identities() -> Vec<(
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
)> {
    for_each_object_type!(identity_lookup)
}

#[test]
fn the_identity_table_covers_every_wrapper_type_exactly_once() {
    let identities = identities();
    assert_eq!(
        identities.len(),
        56,
        "The table must cover all 56 wrapper types."
    );
    let names = identities
        .iter()
        .map(|(type_name, ..)| *type_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), 56, "Two rows share a type name.");

    // Every element name is recognized by at least one type of its namespace, and every
    // namespace that appears is one of the namespaces this crate supports.
    let supported = BTreeSet::from([
        NS_SBML_CORE.1,
        NS_LAYOUT.1,
        NS_QUAL.1,
        NS_FBC.1,
        crate::constants::namespaces::URL_MATHML,
    ]);
    for (type_name, _, _, namespace) in identities.iter() {
        assert!(
            supported.contains(namespace),
            "{} declares the namespace '{}', which the crate does not know.",
            type_name,
            namespace
        );
    }
}

/// The five types whose element has more than one name, with the evidence for each name.
///
/// The names are the de-capitalized class names the qual and layout specifications prescribe for
/// the children of their lists (`sbml-level-3/version-1/layout/syntax.tex` for the glyphs); the test
/// below pins that each declaration matches this list, so a name cannot be dropped or added
/// silently. `Point` is the least obvious member: the layout package writes it as `<position>`,
/// `<start>`, `<end>`, `<basePoint1>` or `<basePoint2>`, never as `<point>`.
///
/// The curve segment classes are the counter-example and are pinned separately by the test below:
/// `LineSegment` and `CubicBezier` share one element name and are told apart by `xsi:type`.
#[test]
fn the_multi_name_types_recognize_exactly_the_names_of_the_specification() {
    let expected: &[(&str, &[&str])] = &[
        (
            "AbstractRule",
            &["algebraicRule", "assignmentRule", "rateRule"],
        ),
        ("AbstractTerm", &["defaultTerm", "functionTerm"]),
        (
            "GraphicalObject",
            &[
                "graphicalObject",
                "compartmentGlyph",
                "generalGlyph",
                "reactionGlyph",
                "referenceGlyph",
                "speciesGlyph",
                "speciesReferenceGlyph",
                "textGlyph",
            ],
        ),
        ("Association", &["geneProductRef", "and", "or"]),
        (
            "Point",
            &["position", "start", "end", "basePoint1", "basePoint2"],
        ),
    ];
    let identities = identities();
    for (type_name, names) in expected {
        let (_, primary, declared, _) = identities
            .iter()
            .find(|(it, ..)| it == type_name)
            .unwrap_or_else(|| panic!("{} is missing from the identity table.", type_name));
        assert_eq!(
            *declared, *names,
            "The recognized element names of {} changed.",
            type_name
        );
        assert_eq!(*primary, names[0]);
    }
    // Every other type recognizes exactly one name, including the two curve segment classes: the
    // layout specification writes both of them as `<layout:curveSegment>` and discriminates them by
    // `xsi:type`, which the dedicated test below pins.
    let multi = expected.iter().map(|(it, _)| *it).collect::<BTreeSet<_>>();
    for (type_name, _, declared, _) in identities.iter() {
        if multi.contains(type_name) {
            continue;
        }
        assert_eq!(
            declared.len(),
            1,
            "{} is not a multi-name type but declares several names.",
            type_name
        );
    }
    for type_name in ["LineSegment", "CubicBezier"] {
        let (_, _, declared, _) = identities
            .iter()
            .find(|(it, ..)| *it == type_name)
            .expect("The curve segment classes are part of the table.");
        assert_eq!(*declared, ["curveSegment"]);
    }
}

/// Every type that has a `default(document)` constructor builds an element its own identity
/// recognizes.
///
/// This is not a tautology: the constructor and the identity are written in different places (the
/// constructor in the type's module, the identity in its attribute), and one of them can be
/// changed without the other. None of the thirteen types below is one of the creation-path
/// mismatches of the worklist, so the check is meaningful.
macro_rules! check_default_identity {
    ($($type:ty),* $(,)?) => {
        $(
            {
                let value = <$type as XmlDefault>::default(document());
                assert!(
                    <$type as SbmlObject>::matches_element(value.xml_element()),
                    "the default {} is not recognized by its own identity: <{}> in '{}'",
                    stringify!($type),
                    value.xml_element().tag_name(),
                    value.xml_element().namespace_url()
                );
                assert!(value.is_element());
            }
        )*
    };
}

#[test]
fn every_default_object_is_recognized_by_its_own_type() {
    check_default_identity!(
        Model,
        Compartment,
        UnitDefinition,
        Unit,
        FunctionDefinition,
        Constraint,
        KineticLaw,
        Event,
        Trigger,
        Priority,
        Delay,
        AlgebraicRule,
        Math,
    );
}

/// The curve segment variants are told apart by `xsi:type`, and the identity agrees with the
/// typed property the crate already reads when it casts a segment.
#[test]
fn the_curve_segment_variants_are_told_apart_by_xsi_type() {
    let document = Sbml::read_path("test-inputs/test-layout/layout_example.xml")
        .expect("The layout example is a readable document.");
    let segments = document
        .xml_element()
        .recursive_child_elements_filtered(|it| it.tag_name().as_str() == "curveSegment");
    assert!(
        segments.len() >= 2,
        "The layout example must contain curve segments, found {}.",
        segments.len()
    );

    // Classify the real segments through the crate's own typed property, and check that the
    // identity of each variant agrees with it.
    let mut bezier = None;
    let mut line = None;
    for segment in segments {
        let typed = LineSegment::from(segment.clone());
        match typed.xsi_type().get_checked() {
            Ok(Some(XsiType::CubicBezier)) => {
                assert!(CubicBezier::matches_element(&segment));
                assert!(CubicBezier::try_from_element(segment.clone()).is_some());
                // A cubic bezier *is* a line segment: the subtype relation has to hold.
                assert!(LineSegment::matches_element(&segment));
                bezier.get_or_insert(segment);
            }
            Ok(Some(XsiType::LineSegment)) => {
                assert!(LineSegment::matches_element(&segment));
                assert!(LineSegment::try_from_element(segment.clone()).is_some());
                assert!(!CubicBezier::matches_element(&segment));
                assert!(CubicBezier::try_from_element(segment.clone()).is_none());
                line.get_or_insert(segment);
            }
            other => panic!("A curve segment must declare its `xsi:type`, found {other:?}."),
        }
    }
    assert!(
        bezier.is_some(),
        "The example must contain a cubic bezier segment."
    );
    assert!(line.is_some(), "The example must contain a line segment.");
}

/// A package type that describes a core element takes the namespace of the element, not the
/// namespace of the package that adds the data to it.
#[test]
fn a_package_type_whose_element_is_a_core_element_takes_the_core_namespace() {
    assert_eq!(FbcSpecies::ELEMENT_NAMESPACE, NS_SBML_CORE);
    assert_eq!(FbcReaction::ELEMENT_NAMESPACE, NS_SBML_CORE);

    let species = element("species", NS_SBML_CORE, None);
    assert!(FbcSpecies::matches_element(&species));
    assert!(Species::matches_element(&species));
    assert!(!FbcSpecies::matches_element(&element(
        "species", NS_FBC, None
    )));
    // ... and the reverse case: a type whose own element belongs to its package.
    let gene_product = element("geneProduct", NS_FBC, None);
    assert!(GeneProduct::matches_element(&gene_product));
    assert_eq!(GeneProduct::ELEMENT_NAMESPACE, NS_FBC);
    assert!(!GeneProduct::matches_element(&element(
        "geneProduct",
        NS_SBML_CORE,
        None
    )));
}

/// MathML is a namespace of its own, so `Math` must not be confused with a core element.
#[test]
fn math_is_in_the_mathml_namespace() {
    let math = element("math", crate::constants::namespaces::NS_MATHML, None);
    assert!(Math::matches_element(&math));
    assert!(!Math::matches_element(&element("math", NS_SBML_CORE, None)));
}

// ---------------------------------------------------------------------------------------------
// The source cross-check
// ---------------------------------------------------------------------------------------------

/// Every `(type, element name)` pair whose element name the *creation path* writes but the type
/// does not recognize.
///
/// These are exactly the entries of the creation-mismatch worklist: the constructor of each type
/// below writes an element name that the reader of the same type does not use (and, in most cases,
/// that does not appear in the SBML specification at all). They are *not* fixed here — the creation
/// path is a separate unit of work — but the check below asserts that the list is exactly this, so
/// an undocumented mismatch fails the test and a fixed one has to be removed from the list.
const KNOWN_CREATION_MISMATCHES: &[(&str, &str)] = &[
    ("BoundingBox", "bounding_box"),
    ("CubicBezier", "cubicBezier"),
    ("GraphicalObject", "graphObject"),
    ("Point", "point"),
    ("SpeciesReferenceGlyph", "speciesRefGlyph"),
];

/// The types that a child accessor can return without the type having a declared identity.
///
/// `XmlElement` is the raw wrapper of an element that is not interpreted as an SBML object (the
/// notes and annotation payloads), so it stands for no single element name and cannot be part of
/// the identity table.
const TYPES_WITHOUT_IDENTITY: &[&str] = &["XmlElement"];

/// The Rust sources of this crate, in a deterministic order.
fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut entries = std::fs::read_dir(root)
        .unwrap_or_else(|error| panic!("Cannot read {}: {error}", root.display()))
        .map(|it| it.expect("A readable directory entry.").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            result.append(&mut rust_sources(&path));
        } else if path.extension().is_some_and(|it| it == "rs") {
            result.push(path);
        }
    }
    result
}

/// Every wrapper type in the tree declares an identity, and the table knows exactly those types.
///
/// The identity of a type is a declaration next to the type, so nothing in the compiler ties the
/// two together: a wrapper added without an `#[sbml_object(..)]` attribute would simply have no
/// identity, and the row-driven tests above would not notice because they iterate the table. This
/// check reads the source instead, so the table and the tree have to agree.
#[test]
fn every_wrapper_type_declares_an_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut derives = 0usize;
    let mut missing = Vec::new();
    for path in rust_sources(&root) {
        let source = std::fs::read_to_string(&path).expect("A readable Rust source.");
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if !line.starts_with("#[derive(") || !line.contains("XmlWrapper") {
                continue;
            }
            derives += 1;
            let next = lines[index + 1..].iter().find(|it| !it.trim().is_empty());
            match next {
                Some(next) if next.starts_with("#[sbml_object(") => {}
                _ => missing.push(format!("{}:{}", path.display(), index + 1)),
            }
        }
    }
    assert!(
        missing.is_empty(),
        "A `#[derive(XmlWrapper)]` type does not declare its identity:\n{}",
        missing.join("\n")
    );
    assert_eq!(
        derives,
        identities().len(),
        "The tree declares {derives} wrapper types but the identity table has {} rows.",
        identities().len()
    );
}

#[test]
fn the_element_names_the_crate_reads_a_type_from_are_declared() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut read_pairs = 0usize;
    let mut created_pairs = 0usize;
    let mut undocumented = Vec::new();
    let mut creation_mismatches = BTreeSet::new();

    for path in rust_sources(&root) {
        let source =
            strip_line_comments(&std::fs::read_to_string(&path).expect("A readable Rust source."));
        let short = path
            .strip_prefix(&root)
            .expect("The path is inside the source tree.")
            .display()
            .to_string();

        // (a) The element name of a single child accessor: `-> ...SbmlChild<T> { ...("name") }`.
        for (type_name, literal) in child_accessors(&source) {
            if TYPES_WITHOUT_IDENTITY.contains(&type_name.as_str()) {
                continue;
            }
            read_pairs += 1;
            if !identities().iter().any(|(it, ..)| *it == type_name) {
                undocumented.push(format!(
                    "{short}: <{literal}> is read into {type_name}, which declares no identity"
                ));
            } else if !declares(&type_name, &literal) {
                undocumented.push(format!("{short}: {type_name} is read from <{literal}>"));
            }
        }

        // (b) The element name the creation path writes: `T::new_empty(document, "name")`.
        for (type_name, literal) in creation_names(&source) {
            created_pairs += 1;
            if declares(&type_name, &literal) {
                continue;
            }
            if KNOWN_CREATION_MISMATCHES.contains(&(type_name.as_str(), literal.as_str())) {
                creation_mismatches.insert((type_name, literal));
            } else {
                undocumented.push(format!("{short}: {type_name} is created as <{literal}>"));
            }
        }
    }

    assert!(
        undocumented.is_empty(),
        "The crate reads or creates a type from an element name its identity does not recognize; \
         either the identity is wrong or the name belongs in KNOWN_CREATION_MISMATCHES:\n{}",
        undocumented.join("\n")
    );
    assert_eq!(
        creation_mismatches,
        KNOWN_CREATION_MISMATCHES
            .iter()
            .map(|(it, name)| (it.to_string(), name.to_string()))
            .collect::<BTreeSet<_>>(),
        "The set of creation-path mismatches changed: fix the list above."
    );
    // The scan must actually have found the sites it is supposed to check, otherwise it would pass
    // vacuously.
    assert!(
        read_pairs >= 8,
        "The child-accessor scan found only {read_pairs} sites."
    );
    assert!(
        created_pairs >= 45,
        "The creation scan found only {created_pairs} sites."
    );
}

/// Returns `true` if `type_name` declares `literal` as one of its element names.
fn declares(type_name: &str, literal: &str) -> bool {
    identities()
        .iter()
        .find(|(it, ..)| *it == type_name)
        .is_some_and(|(_, _, names, _)| names.contains(&literal))
}

#[test]
fn the_lookup_only_knows_the_wrapper_types() {
    // A guard for the scan above: an element name that belongs to no wrapper type (a list
    // container, a MathML element) must not be reported as recognized, otherwise the scan would
    // accept anything.
    assert!(!declares("Species", "listOfSpecies"));
    assert!(!declares("NotAType", "species"));
    assert!(declares("Species", "species"));
}

/// The element names of the single-child accessors in `source`, as `(wrapper type, name)` pairs.
///
/// Removes the line comments of a Rust source, including the documentation comments.
///
/// The scan below looks for name literals in the *code*, and the documentation of this crate
/// quotes such literals (and URLs, which a naive scan would also cut in half); cutting every line
/// at its `//` is enough for that and keeps the scan from matching its own examples.
fn strip_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `(type, name)` pairs of the single-child accessors of a Rust source.
///
/// Only the accessors whose child is a single object are considered: for an accessor that returns a
/// list (`...SbmlChild<XmlList<T>>`) the literal names the *list container*, which is not the
/// element name of any wrapper type, and the children of the list are covered by the creation scan
/// and by the specification-cited rows of the table instead.
fn child_accessors(source: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    // `pub fn x(&self) -> OptionalSbmlChild<T> { ...("name" ...) }`
    let pattern = regex::Regex::new(
        r#"->\s*(?:Optional|Required)SbmlChild<(\w+)>\s*\{(?:[^{}]|\{[^{}]*\})*?[\w:]+\(\s*(?:[^,()]*,\s*)?"([^"]+)""#,
    )
    .expect("The accessor pattern is a valid regular expression.");
    for captures in pattern.captures_iter(source) {
        result.push((
            captures.get(1).expect("Group 1.").as_str().to_string(),
            captures.get(2).expect("Group 2.").as_str().to_string(),
        ));
    }
    result
}

/// The `T::new_empty(document, "name")` sites in `source`, as `(type, name)` pairs.
fn creation_names(source: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let pattern = regex::Regex::new(r#"(\w+)::new_empty\(\s*document,\s*"([^"]+)"\s*\)"#)
        .expect("The creation pattern is a valid regular expression.");
    for captures in pattern.captures_iter(source) {
        result.push((
            captures.get(1).expect("Group 1.").as_str().to_string(),
            captures.get(2).expect("Group 2.").as_str().to_string(),
        ));
    }
    result
}
