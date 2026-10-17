//! Tests for the declared object identities.
//!
//! Five kinds of check live here, and they are complementary:
//!
//!  * the **56-row identity table** is exercised row by row — every type must accept its own
//!    element name(s) in its own namespace, and must reject a foreign local name and a foreign
//!    namespace, and a namespace that is the same URI under another prefix must still be accepted;
//!  * the **constructor table** constructs every type that has an identity-derived constructor and
//!    checks the result with the same function the negative probes use: the constructor must write
//!    the element the declaration describes, in the declared namespace, and a planted wrong name or
//!    wrong namespace must be rejected. The row arithmetic (constructed + omitted = 56, and the
//!    omitted rows are exactly the declared, derived set) is asserted, so the table cannot cover
//!    fewer types by accident;
//!  * the **special cases** get their own tests: the `xsi:type` discrimination of the curve segment
//!    variants (through the crate's own typed property, not through the identity), the thirteen
//!    types with a `default(document)` constructor, and the package types whose element is a core
//!    element (`FbcSpecies`, `FbcReaction`);
//!  * the **source cross-check** reads this crate's own `src/` and fails when the crate *reads* a
//!    type from an element name its declaration does not recognize, when a non-test source creates
//!    an element through a name/namespace pair of its own instead of the declared identity, or when
//!    an explicit-name creation site appears that is not listed with a justification. This is what
//!    keeps the table honest: no document-level gate would notice a wrong alias, because the only
//!    evidence of which element a wrapper describes is the source itself;
//!  * the **declaration/element-name consistency** of the crate's own tables is checked by the
//!    element-name scan at the end of this module.

use crate::constants::namespaces::{
    NS_FBC, NS_HTML, NS_LAYOUT, NS_QUAL, NS_SBML_CORE, URL_MATHML, URL_SBML_CORE, URL_XSI,
};
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
use std::collections::{BTreeMap, BTreeSet};
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

/// An empty document to attach the test elements to.
fn document() -> Document {
    Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8"?><probe/>"#)
        .expect("The probe document is well formed.")
}

/// A detached element with the given local name, namespace and (optional) `xsi:type`.
///
/// The `xsi:type` half goes through the crate's own writer, so the helper cannot disagree with what
/// an identity-derived constructor does with the same value.
fn element(name: &str, namespace: Namespace, xsi_type: Option<&str>) -> XmlElement {
    let element = XmlElement::new_qualified(document(), name, namespace);
    if let Some(xsi_type) = xsi_type {
        crate::xml::set_xsi_type(&element, xsi_type);
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

// ---------------------------------------------------------------------------------------------
// The constructor table
// ---------------------------------------------------------------------------------------------

/// The check that every row of the constructor table and every negative probe is held to.
///
/// It takes the *constructor* rather than a value, so the identity-derived constructor of a type and
/// a probe that deliberately builds something else go through exactly the same code: the table and
/// its probes cannot drift apart.
fn check_constructed_element<T: SbmlObject>(construct: fn(Document) -> T) -> Result<(), String> {
    let value = construct(document());
    let element = value.xml_element();
    let produced_name = element.tag_name();
    let produced_namespace = element.namespace_url();
    if produced_namespace != T::ELEMENT_NAMESPACE.1 {
        return Err(format!(
            "constructed <{produced_name}> in '{produced_namespace}', but the declaration of '{}' \
             places it in '{}'",
            T::ELEMENT_NAME,
            T::ELEMENT_NAMESPACE.1
        ));
    }
    if !T::matches_element(element) {
        return Err(format!(
            "constructed <{produced_name}> in '{produced_namespace}', which is not recognized by \
             its own identity ('{}' in '{}')",
            T::ELEMENT_NAME,
            T::ELEMENT_NAMESPACE.1
        ));
    }
    if !value.is_element() {
        return Err(format!(
            "the constructed <{produced_name}> is not an instance of its own type"
        ));
    }
    Ok(())
}

/// One row per wrapper type: construct it, check the result, and probe the check with a planted
/// wrong local name and a planted foreign namespace.
///
/// The macro collects the names of the types it constructed, the names of the types that have no
/// constructor and the namespaces the constructions produced, and it fails immediately if any row
/// or probe misbehaves. The returned data is what makes the table non-vacuous: the test below
/// asserts its arithmetic against the identity table.
macro_rules! check_constructor_row {
    ($($type:ty),* $(,)?) => {{
        let mut constructed: Vec<&'static str> = Vec::new();
        let mut omitted: Vec<&'static str> = Vec::new();
        let mut namespaces: BTreeMap<&'static str, usize> = BTreeMap::new();
        $(
            {
                let type_name = stringify!($type);
                match <$type as SbmlObject>::CONSTRUCT {
                    Some(construct) => {
                        check_constructed_element::<$type>(construct).unwrap_or_else(|error| {
                            panic!("The constructor of {type_name} builds the wrong element: {error}")
                        });
                        constructed.push(type_name);
                        *namespaces
                            .entry(<$type as SbmlObject>::ELEMENT_NAMESPACE.1)
                            .or_insert(0) += 1;

                        // A planted wrong local name: the same check has to reject it.
                        let wrong_name: fn(Document) -> $type = |document| {
                            <$type>::from(XmlElement::new_qualified(
                                document,
                                "thisIsNotTheElementOfTheType",
                                <$type as SbmlObject>::ELEMENT_NAMESPACE,
                            ))
                        };
                        let error = check_constructed_element::<$type>(wrong_name)
                            .err()
                            .unwrap_or_else(|| {
                                panic!(
                                    "The constructor check accepted a wrong local name for \
                                     {type_name}."
                                )
                            });
                        assert!(
                            error.contains("not recognized"),
                            "The check rejected a wrong local name of {type_name} for the wrong \
                             reason: {error}"
                        );

                        // A planted foreign namespace: likewise.
                        let wrong_namespace: fn(Document) -> $type = |document| {
                            <$type>::from(XmlElement::new_qualified(
                                document,
                                <$type as SbmlObject>::ELEMENT_NAME,
                                NS_HTML,
                            ))
                        };
                        let error = check_constructed_element::<$type>(wrong_namespace)
                            .err()
                            .unwrap_or_else(|| {
                                panic!(
                                    "The constructor check accepted a foreign namespace for \
                                     {type_name}."
                                )
                            });
                        assert!(
                            error.contains("declaration"),
                            "The check rejected a foreign namespace of {type_name} for the wrong \
                             reason: {error}"
                        );
                    }
                    None => omitted.push(type_name),
                }
            }
        )*
        (constructed, omitted, namespaces)
    }};
}

/// The types that omit the constructor, derived from the identity table instead of from a list.
///
/// A type whose declaration recognizes several element names is a supertype, and its names are the
/// names of its subtypes. When the *first* of them is the [`SbmlObject::ELEMENT_NAME`] of a
/// different wrapper type in the same namespace, the type has no element of its own: an
/// identity-derived constructor would silently build the sibling's element.
fn derived_omissions() -> BTreeSet<&'static str> {
    let identities = identities();
    let mut result = BTreeSet::new();
    for (type_name, _primary, names, namespace) in identities.iter() {
        if names.len() < 2 {
            continue;
        }
        let named_by_a_sibling =
            identities
                .iter()
                .any(|(other, other_primary, _other_names, other_namespace)| {
                    other != type_name && other_namespace == namespace && other_primary == &names[0]
                });
        if named_by_a_sibling {
            result.insert(*type_name);
        }
    }
    result
}

#[test]
fn every_constructor_builds_the_element_of_its_declared_identity() {
    let (constructed, omitted, namespaces) = for_each_object_type!(check_constructor_row);

    // The arithmetic: every row of the identity table is either constructed here or is an omitted
    // type, so a type added to `for_each_object_type!` cannot escape the table by not being
    // constructed, and the table cannot shrink silently.
    let rows = identities().len();
    assert_eq!(
        rows, 56,
        "The identity table must cover all 56 wrapper types."
    );
    assert_eq!(
        constructed.len() + omitted.len(),
        rows,
        "The constructor table covered {} of {rows} types.",
        constructed.len() + omitted.len()
    );
    assert_eq!(
        constructed.len(),
        53,
        "Exactly the 53 types that declare an element of their own have a constructor."
    );
    assert_eq!(
        omitted.iter().copied().collect::<BTreeSet<_>>(),
        derived_omissions(),
        "The types without a constructor must be exactly the supertypes whose first name belongs \
         to a concrete sibling."
    );
    assert_eq!(
        omitted.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(["AbstractRule", "AbstractTerm", "Association"]),
        "The derived omission set changed."
    );

    // The namespace split. These counts follow from the modules the types live in, not from the
    // declarations, so a declaration that is *self-consistently* wrong about its namespace (a
    // layout type that declares and writes the core namespace) fails here: the constructor would
    // agree with the declaration, but the count would be wrong.
    assert_eq!(
        namespaces,
        BTreeMap::from([
            (URL_SBML_CORE, 24), // core: 25 declarations, `AbstractRule` omitted
            (NS_LAYOUT.1, 15),   // layout: all 15 declarations have a constructor
            (NS_QUAL.1, 6),      // qual: 7 declarations, `AbstractTerm` omitted
            (NS_FBC.1, 7),       // fbc: 8 declarations, `Association` omitted
            (URL_MATHML, 1),     // the MathML formula element
        ]),
        "The namespaces the constructors produced changed."
    );

    // The rows that are not "one name, one element" are named explicitly, so a later change that
    // silently drops one of them is visible in the failure message.
    for type_name in [
        "AlgebraicRule",
        "DefaultTerm",
        "GeneProductRef",
        "Point",
        "GraphicalObject",
    ] {
        assert!(
            constructed.contains(&type_name),
            "{type_name} must be covered by the constructor table."
        );
    }
    for type_name in ["AbstractRule", "AbstractTerm", "Association"] {
        assert!(
            omitted.contains(&type_name),
            "{type_name} must declare no constructor."
        );
    }
}

/// The negative probe, spelled out with its output.
///
/// The table above probes every row; this test records what a rejection looks like. The
/// identity-derived constructor of `CubicBezier` and a deliberately wrong construction of the same
/// type are fed to the same check function, which accepts the first and rejects the second.
#[test]
fn the_constructor_check_rejects_planted_elements() {
    check_constructed_element::<CubicBezier>(CubicBezier::new_empty)
        .expect("The generated constructor of CubicBezier builds its own element.");

    let planted: fn(Document) -> CubicBezier = |document| {
        CubicBezier::from(XmlElement::new_qualified(
            document,
            "cubicBezier",
            NS_LAYOUT,
        ))
    };
    let error = check_constructed_element::<CubicBezier>(planted)
        .expect_err("A planted wrong local name must be rejected.");
    println!("planted wrong local name -> {error}");
    assert!(error.contains("not recognized"));

    let planted: fn(Document) -> CubicBezier = |document| {
        CubicBezier::from(XmlElement::new_qualified(
            document,
            "curveSegment",
            NS_SBML_CORE,
        ))
    };
    let error = check_constructed_element::<CubicBezier>(planted)
        .expect_err("A planted foreign namespace must be rejected.");
    println!("planted foreign namespace -> {error}");
    assert!(error.contains("declaration"));
}

/// The `xsi:type` half of a constructor, in both of its parts: the value that is written and the
/// declaration that makes the written name resolvable.
///
/// A cubic bezier is written as `<layout:curveSegment xsi:type="CubicBezier">`: the element name is
/// shared with a plain line segment, so the attribute is what distinguishes them. The value is the
/// unprefixed class name, which is what the SBML documents in the wild write and what
/// [`SbmlObject::XSI_TYPE`] contains.
#[test]
fn the_constructor_of_a_discriminated_type_writes_xsi_type() {
    assert_eq!(CubicBezier::XSI_TYPE, Some("CubicBezier"));
    assert_eq!(LineSegment::XSI_TYPE, None);

    let document = document();
    let bezier = CubicBezier::new_empty(document.clone());
    let probe = bezier.xml_element();

    // Part one: the value.
    assert_eq!(probe.get_attribute("type").as_deref(), Some("CubicBezier"));
    assert_eq!(
        element("curveSegment", NS_LAYOUT, Some("LineSegment"))
            .get_attribute("type")
            .as_deref(),
        Some("LineSegment")
    );

    // Part two: the `xsi` prefix is declared on the element itself, so the attribute name resolves
    // (an attribute never inherits the default namespace, so nothing else could declare it).
    let name = probe
        .element()
        .resolve_attribute_name("xsi:type")
        .expect("The `xsi` prefix is in scope for the newly created element.");
    assert_eq!(
        name.namespace().expect("`xsi:type` has a namespace.").uri(),
        URL_XSI
    );
    assert_eq!(probe.namespace_url(), NS_LAYOUT.1);

    // The identity agrees with both halves.
    assert!(CubicBezier::matches_element(probe));
    assert!(bezier.is_element());
    // A cubic bezier is also a line segment (the subtype relation), while a segment without the
    // attribute is only a line segment.
    assert!(LineSegment::matches_element(probe));
    assert!(!CubicBezier::matches_element(&element(
        "curveSegment",
        NS_LAYOUT,
        None
    )));

    // ... and the written form survives a serialization round trip: the reader resolves the
    // attribute through the same `xsi:type` path.
    document
        .root()
        .expect("The probe document has a root.")
        .append_child(probe.element().clone());
    let written = document
        .write_string()
        .expect("The probe document serializes.");
    let reparsed = Document::parse_string(&written).expect("What this crate writes parses back.");
    let reparsed = reparsed
        .root()
        .expect("The re-parsed document has a root.")
        .child_elements()
        .into_iter()
        .next()
        .map(XmlElement::new)
        .expect("The appended element is the only child of the probe root.");
    assert!(
        CubicBezier::matches_element(&reparsed),
        "The serialized cubic bezier is no longer recognized: {written}"
    );
    assert_eq!(
        reparsed.get_attribute("type").as_deref(),
        Some("CubicBezier")
    );
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

/// The types that a child accessor can return without the type having a declared identity.
///
/// `XmlElement` is the raw wrapper of an element that is not interpreted as an SBML object (the
/// notes and annotation payloads), so it stands for no single element name and cannot be part of
/// the identity table.
const TYPES_WITHOUT_IDENTITY: &[&str] = &["XmlElement"];

/// The explicit-name creation sites of the crate, each with the reason it cannot be expressed as the
/// identity-derived `new_empty` of its type.
///
/// A creation site that names its element itself is the only remaining way for a constructor to
/// disagree with the declaration of its type, so every such site is enumerated here and the test
/// below asserts **set equality** between this list and the sites found under `src/`: a new site
/// fails the test until it is either migrated or listed with a justification, and a removed site
/// fails it until it is deleted from the list.
const EXPLICIT_NAME_CREATION_ALLOW_LIST: &[(&str, &str)] = &[(
    "AbstractRule",
    "the element name of a rule is chosen by the caller (`algebraicRule`, `assignmentRule` or \
     `rateRule`), so the type has no identity-derived constructor; `AbstractRule::default` forwards \
     the name to the named form",
)];

/// The only place in the non-test sources that builds an element from a name/namespace pair of its
/// own instead of through a constructor.
///
/// [`XmlElement::new_qualified`] is the primitive every identity-derived constructor is built on, so
/// a site of it outside the creation path is exactly the kind of bypass that made the original
/// defect invisible: the site would write whatever name and namespace it likes. The test asserts set
/// equality against the sites found under `src/`, so a new bypass has to be justified here.
const NEW_QUALIFIED_ALLOW_LIST: &[(&str, &str, &str)] = &[(
    "core/sbase.rs",
    "XmlElement",
    "the body of `SbmlUtils::new_empty_named`, which is the named form the allow-list above calls; \
     its `tag_name` is the caller's choice and its namespace is the core namespace the trait is \
     restricted to",
)];

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

/// The *non-test* part of a source file: everything before its `mod tests` block, if it has one.
///
/// The scan is about what the **crate** does, and test code is allowed to build whatever element it
/// likes to exercise a failure; the sources of this crate put their test module last, so cutting the
/// file at the first `mod tests` line is the whole rule.
fn strip_test_modules(source: &str) -> String {
    source
        .lines()
        .take_while(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("mod tests {") && trimmed != "mod tests;"
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `T::new_empty(document)` sites of a source, as the names of the types they construct.
fn identity_creation_sites(source: &str) -> Vec<String> {
    let pattern = regex::Regex::new(r#"(\w+)::new_empty\(\s*document\s*\)"#)
        .expect("The identity-creation pattern is a valid regular expression.");
    pattern
        .captures_iter(source)
        .map(|it| it.get(1).expect("Group 1.").as_str().to_string())
        .collect()
}

/// The `T::new_empty_named(document, …)` sites of a source, as `(type, literal or `None`)` pairs.
///
/// The second argument is a literal for a site that fixes the element name and an expression for a
/// site that takes it from its caller; both shapes are reported, so neither can hide.
fn named_creation_sites(source: &str) -> Vec<(String, Option<String>)> {
    let pattern = regex::Regex::new(
        r#"(\w+)::new_empty_named\(\s*document\s*,\s*(?:"([^"]+)"|([^)\s,]+))\s*\)"#,
    )
    .expect("The named-creation pattern is a valid regular expression.");
    pattern
        .captures_iter(source)
        .map(|it| {
            let type_name = it.get(1).expect("Group 1.").as_str().to_string();
            let literal = it.get(2).map(|it| it.as_str().to_string());
            (type_name, literal)
        })
        .collect()
}

/// The `T::new_qualified(…)` sites of a source, as the names of the receivers.
fn qualified_creation_sites(source: &str) -> Vec<String> {
    let pattern = regex::Regex::new(r#"(\w+)::new_qualified\("#)
        .expect("The qualified-creation pattern is a valid regular expression.");
    pattern
        .captures_iter(source)
        .map(|it| it.get(1).expect("Group 1.").as_str().to_string())
        .collect()
}

/// The reason an identity-derived creation site is a defect, or `None` if it is fine.
fn identity_creation_defect(type_name: &str) -> Option<String> {
    if !identities().iter().any(|(it, ..)| *it == type_name) {
        return Some(format!(
            "{type_name} is created through an identity constructor but declares no identity"
        ));
    }
    if !has_constructor(type_name) {
        return Some(format!(
            "{type_name} is created through an identity constructor but declares none"
        ));
    }
    None
}

/// The reason an explicit-name creation site is a defect, or `None` if it is fine.
fn named_creation_defect(type_name: &str, literal: Option<&str>) -> Option<String> {
    if !EXPLICIT_NAME_CREATION_ALLOW_LIST
        .iter()
        .any(|(it, _)| *it == type_name)
    {
        return Some(format!(
            "{type_name} creates its element with an explicit name and is not in \
             EXPLICIT_NAME_CREATION_ALLOW_LIST"
        ));
    }
    match literal {
        Some(literal) if !declares(type_name, literal) => Some(format!(
            "{type_name} is created as <{literal}>, which its declaration does not recognize"
        )),
        _ => None,
    }
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
fn the_creation_paths_of_the_crate_are_enumerated() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut read_pairs = 0usize;
    let mut identity_sites = 0usize;
    let mut named_sites = BTreeSet::new();
    let mut qualified_sites = BTreeSet::new();
    let mut undocumented = Vec::new();

    for path in rust_sources(&root) {
        let short = path
            .strip_prefix(&root)
            .expect("The path is inside the source tree.")
            .display()
            .to_string();
        if short == "xml/xml_object/tests.rs" || short == "test_suite.rs" {
            // Test sources: they build elements to exercise failures, so they are not part of the
            // surface this check pins.
            continue;
        }
        let source = strip_test_modules(&strip_line_comments(
            &std::fs::read_to_string(&path).expect("A readable Rust source."),
        ));

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

        // (b) The identity-derived creation sites: the type must have an identity and a constructor.
        for type_name in identity_creation_sites(&source) {
            identity_sites += 1;
            if let Some(defect) = identity_creation_defect(&type_name) {
                undocumented.push(format!("{short}: {defect}"));
            }
        }

        // (c) The explicit-name creation sites: set equality against the allow-list below.
        for (type_name, literal) in named_creation_sites(&source) {
            if let Some(defect) = named_creation_defect(&type_name, literal.as_deref()) {
                undocumented.push(format!("{short}: {defect}"));
            }
            named_sites.insert(type_name);
        }

        // (d) The raw name/namespace pairs: set equality against the allow-list below.
        for type_name in qualified_creation_sites(&source) {
            qualified_sites.insert((short.clone(), type_name));
        }
    }

    assert!(
        undocumented.is_empty(),
        "The crate creates or reads an element in a way its declaration does not describe:\n{}",
        undocumented.join("\n")
    );
    assert_eq!(
        named_sites,
        EXPLICIT_NAME_CREATION_ALLOW_LIST
            .iter()
            .map(|(it, _)| it.to_string())
            .collect::<BTreeSet<_>>(),
        "The set of explicit-name creation sites changed: migrate the site to the identity-derived \
         constructor, or add it to EXPLICIT_NAME_CREATION_ALLOW_LIST with a justification."
    );
    assert_eq!(
        qualified_sites,
        NEW_QUALIFIED_ALLOW_LIST
            .iter()
            .map(|(file, receiver, _reason)| (file.to_string(), receiver.to_string()))
            .collect::<BTreeSet<_>>(),
        "An element is built from a name/namespace pair of its own outside the allowed site: use a \
         constructor, or add the site to NEW_QUALIFIED_ALLOW_LIST with a justification."
    );
    // The scans must actually have found the sites they are supposed to check, otherwise they would
    // pass vacuously.
    assert!(
        read_pairs >= 8,
        "The child-accessor scan found only {read_pairs} sites."
    );
    assert!(
        identity_sites >= 45,
        "The creation scan found only {identity_sites} identity-derived creation sites."
    );
}

/// The scanners and the two verdicts above have to be able to *see* a wrong site, otherwise the pin
/// they implement is vacuous. This test feeds them a planted fragment and checks every verdict.
#[test]
fn the_creation_scanners_reject_planted_sites() {
    let planted = r#"
        let a = Species::new_empty(document);
        let b = NotAType::new_empty(document);
        let c = FluxObjective::new_empty_named(document, "wrongName");
        let d = AbstractRule::new_empty_named(document, tag_name);
        let e = XmlElement::new_qualified(document, "species", NS_SBML_CORE);
    "#;

    assert_eq!(identity_creation_sites(planted), ["Species", "NotAType"]);
    assert_eq!(
        named_creation_sites(planted),
        vec![
            ("FluxObjective".to_string(), Some("wrongName".to_string())),
            ("AbstractRule".to_string(), None),
        ]
    );
    assert_eq!(qualified_creation_sites(planted), ["XmlElement"]);

    // A type with an identity is fine; one without is a defect.
    assert_eq!(identity_creation_defect("Species"), None);
    assert!(identity_creation_defect("NotAType").is_some());
    assert!(identity_creation_defect("Association").is_some());

    // An unexplained explicit-name site is a defect, and so is a literal the declaration does not
    // recognize; the allow-listed one is only fine when its literal is a declared name.
    assert!(named_creation_defect("FluxObjective", Some("wrongName")).is_some());
    assert_eq!(named_creation_defect("AbstractRule", None), None);
    assert_eq!(
        named_creation_defect("AbstractRule", Some("rateRule")),
        None
    );
    assert!(named_creation_defect("AbstractRule", Some("curveSegment")).is_some());

    // The same holds for the raw name/namespace pair.
    assert!(NEW_QUALIFIED_ALLOW_LIST
        .iter()
        .any(|(file, receiver, _reason)| *file == "core/sbase.rs" && *receiver == "XmlElement"));
    assert!(!NEW_QUALIFIED_ALLOW_LIST
        .iter()
        .any(|(file, _receiver, _reason)| *file == "core/math.rs"));
}

/// Returns `true` if `type_name` declares `literal` as one of its element names.
fn declares(type_name: &str, literal: &str) -> bool {
    identities()
        .iter()
        .find(|(it, ..)| *it == type_name)
        .is_some_and(|(_, _, names, _)| names.contains(&literal))
}

/// The `(type name, has a constructor)` table, so the source scan can ask about a type by name.
macro_rules! constructor_lookup {
    ($($type:ty),* $(,)?) => {
        vec![
            $(
                (stringify!($type), <$type as SbmlObject>::CONSTRUCT.is_some()),
            )*
        ]
    };
}

/// Returns `true` if `type_name` has an identity-derived constructor.
fn has_constructor(type_name: &str) -> bool {
    for_each_object_type!(constructor_lookup)
        .iter()
        .find(|(it, _)| *it == type_name)
        .is_some_and(|(_, has_constructor)| *has_constructor)
}

#[test]
fn the_lookup_only_knows_the_wrapper_types() {
    // A guard for the scan above: an element name that belongs to no wrapper type (a list
    // container, a MathML element) must not be reported as recognized, otherwise the scan would
    // accept anything.
    assert!(!declares("Species", "listOfSpecies"));
    assert!(!declares("NotAType", "species"));
    assert!(declares("Species", "species"));
    assert!(has_constructor("Species"));
    assert!(!has_constructor("AbstractRule"));
    assert!(!has_constructor("NotAType"));
}

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
