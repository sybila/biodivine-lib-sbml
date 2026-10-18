//! The version-aware element tables: the base tables of [`crate::constants::element`] plus the
//! per-version deltas, behind one accessor per table.
//!
//! ## Why a delta, and not one table per version
//!
//! The tables of the validation layer describe one version of one level of SBML, and the crate
//! models more than one of them. Writing a full table per version would mean maintaining several
//! hand-written copies of the same ~90 rows, and the differences between two versions of one level
//! are a handful of names: a copy would drift silently, because nothing would compare it against
//! the version it was copied from, and the element-name completeness scan of
//! `src/xml/xml_object/tests.rs` would see only whichever copy it was pointed at.
//!
//! The tables of [`crate::constants::element`] are therefore the **single base table** (the Level 3
//! Version 2 core and package table, which is the version this crate writes with), and every other
//! version is a [`NameDelta`] against it: the names it *adds* to a base row and the names it
//! *removes* from it. A row that a version does not mention is unchanged, which is the common case
//! and the reason a delta stays readable.
//!
//! ## The derivation, and what a delta row has to carry
//!
//! The authority for a version difference is the specification's **per-rule validation text**:
//! `sbml-level-3/version-1/core/spec/validation-rules/*.tex` for Version 1 and
//! `sbml-level-3/version-2/core/spec/validation-rules/*.tex` for Version 2, whose files carry the
//! version markers (`\verdiff{..}{..}`, `\verone{..}`, `\vertwo{..}`, `\changed{..}`,
//! `\removedRule{..}{}`). Those texts state the *complete* attribute set of an element ("A `Unit`
//! object must have the required attributes `kind`, `exponent`, `scale` and `multiplier`, and may
//! have the optional attributes ...  No other attributes from the SBML Level 3 Core namespace are
//! permitted on a `Unit` object"), which is what makes them the right source: the class diagrams in
//! `components.tex` mark only the *optional* attributes, so reading required-ness off a diagram is
//! wrong in both directions.
//!
//! Every delta row of this module therefore names the rule id it comes from in a comment, and the
//! number of corpus documents that exercise it is recorded in the report of the unit that added it.
//! A row without corpus coverage is legitimate - it states what the specification says - but it has
//! to say so.
//!
//! ## The shape of the accessors
//!
//! Each accessor answers "what does this table say about this element in this version", and
//! returns [`Cow`] so that the common case (a row no delta mentions) borrows the base table and
//! allocates nothing. The version is resolved once per element by
//! [`document_level_version_or_base`](crate::constants::version::document_level_version_or_base),
//! which reads the root of the element's document.

use std::borrow::Cow;

use phf::{phf_map, Map};

use crate::constants::element::{
    ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES, MATHML_ALLOWED_CHILDREN,
    REQUIRED_ATTRIBUTES, REQUIRED_CHILDREN, UNIQUE_CHILDREN,
};
use crate::constants::version::LevelVersion;

/// A table row's difference from the base table, for one group of versions.
///
/// `added` names what the versions have and the base table does not; `removed` names what the base
/// table has and the versions do not. Both are usually tiny, and either may be empty.
#[derive(Debug, Copy, Clone)]
pub(crate) struct NameDelta {
    /// The versions this delta applies to. A version not listed here uses the base row as it is.
    pub versions: &'static [LevelVersion],
    /// The names these versions add to the base row.
    pub added: &'static [&'static str],
    /// The names these versions remove from the base row.
    pub removed: &'static [&'static str],
}

/// A row of the attribute-type table's difference from the base table.
#[derive(Debug, Copy, Clone)]
pub(crate) struct TypeDelta {
    /// The versions this delta applies to.
    pub versions: &'static [LevelVersion],
    /// The `(attribute, type)` pairs these versions add to the base row.
    pub added: &'static [(&'static str, &'static str)],
    /// The attributes these versions remove from the base row.
    pub removed: &'static [&'static str],
}

/// The version group of a Level 3 Version 1-only delta, spelled once so the rows stay one line.
const L3V1: &[LevelVersion] = &[LevelVersion::L3V1];

/// The elements whose Level 3 Version 1 text lists only `metaid` and `sboTerm` as attributes.
///
/// The list is the *expectation* the test `the_attributes_version_2_added_are_absent_in_version_1`
/// compares the delta table against; the table is the authority for what the crate does, and the
/// rule ids below are the derivation of the list.
///
/// Version 2 adds the optional `id` and `name` attributes to these elements, so in Version 1 they
/// are *not* permitted; the crate's base table allows both everywhere (it applies
/// `ALLOWED_SBASE_ATTRIBUTES` to every element), which is why the delta has to remove them.
///
/// The evidence is one rule file per element, all of them of the form "X object may have the
/// optional attributes `metaid` and `sboTerm`. No other attributes from the SBML Level 3 Core
/// namespace are permitted on an X object", with Version 2 adding `\vertwo{\token{id},
/// \token{name}, }` (or, for `eventAssignment`, the two names directly):
///
/// | elements | rule ids |
/// | --- | --- |
/// | `sbml` | 20108 |
/// | the ten `listOf*` containers of a `Model` | 20223-20232 |
/// | `listOfUnits`, `unit` | 20420, 20421 |
/// | `initialAssignment` | 20805 |
/// | `assignmentRule`, `rateRule`, `algebraicRule` | 20908, 20909, 20910 |
/// | `constraint` | 21009 |
/// | `listOfLocalParameters`, `kineticLaw` | 21129, 21132 |
/// | `listOfReactants`, `listOfProducts`, `listOfModifiers` | 21150, 21151 |
/// | `eventAssignment`, `listOfEventAssignments` | 21214, 21224 |
/// | `trigger`, `delay`, `priority` | 21226, 21227, 21232 |
///
/// `localParameter` (21172) and `event` (21225) are deliberately absent: their Version 1 texts list
/// `name` (and `localParameter` lists `name`, `value` and `units`), so Version 1 permits what the
/// base table permits and there is nothing to remove.
#[cfg(test)]
const SBASE_ATTRIBUTES_WITHOUT_ID_AND_NAME: &[&str] = &[
    "sbml",
    "listOfFunctionDefinitions",
    "listOfUnitDefinitions",
    "listOfCompartments",
    "listOfSpecies",
    "listOfParameters",
    "listOfInitialAssignments",
    "listOfRules",
    "listOfConstraints",
    "listOfReactions",
    "listOfEvents",
    "listOfUnits",
    "unit",
    "initialAssignment",
    "assignmentRule",
    "rateRule",
    "algebraicRule",
    "constraint",
    "listOfLocalParameters",
    "kineticLaw",
    "listOfReactants",
    "listOfProducts",
    "listOfModifiers",
    "eventAssignment",
    "listOfEventAssignments",
    "trigger",
    "delay",
    "priority",
];

/// The names Version 2 added to the elements listed below.
const ID_AND_NAME: &[&str] = &["id", "name"];

/// The per-element deltas of the allowed-attribute table.
///
/// Two rows, both from the per-rule texts of Level 3:
///
///  * `reaction`: rule 21110 of Version 1 reads "A `Reaction` object must have the required
///    attributes `id`, `reversible` and `fast`, and may have the optional attributes `metaid`,
///    `sboTerm`, `name` and `compartment`", while Version 2 reads "must have the required
///    attributes `id` and `reversible`" (`\verdiff{id, reversible and fast}{id and reversible}`)
///    and has a paragraph "The `fast` attribute (removed)". The attribute is therefore a Version 1
///    addition. It is the largest class of the unit: 1721 corpus reports of rule `21110` are this
///    one name;
///  * the elements whose Version 1 text lists only `metaid` and `sboTerm` (the rule ids in
///    the table below), which lose `id` and `name`.
pub(crate) static ALLOWED_ATTRIBUTE_DELTAS: Map<&str, &[NameDelta]> = phf_map! {
    "reaction" => &[NameDelta { versions: L3V1, added: &["fast"], removed: &[] }],
    "sbml" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfFunctionDefinitions" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfUnitDefinitions" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfCompartments" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfSpecies" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfParameters" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfInitialAssignments" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfRules" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfConstraints" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfReactions" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfEvents" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfUnits" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "unit" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "initialAssignment" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "assignmentRule" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "rateRule" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "algebraicRule" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "constraint" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfLocalParameters" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "kineticLaw" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfReactants" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfProducts" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfModifiers" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "eventAssignment" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "listOfEventAssignments" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "trigger" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "delay" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
    "priority" => &[NameDelta { versions: L3V1, added: &[], removed: ID_AND_NAME }],
};

/// The per-element deltas of the required-attribute table.
///
/// One row: rule 21110 of Version 1 requires `fast` on a `Reaction`, so a Version 1 document whose
/// reaction has no `fast` attribute is invalid (`syntactic/21110/21110-fail-01-03-sev2-l3v1.xml`
/// is exactly that document). Version 2 removed the requirement with the attribute.
///
/// There is no *relaxation* in the other direction: the required-attribute sets of the two versions
/// are otherwise identical, which is worth stating because the class diagrams suggest the opposite
/// (Version 1 marks only optional attributes, so `id: SId` without a marker means *required*, and
/// `components.tex` of Version 1 says "`Compartment` has one required attribute, `id`").
pub(crate) static REQUIRED_ATTRIBUTE_DELTAS: Map<&str, &[NameDelta]> = phf_map! {
    "reaction" => &[NameDelta { versions: L3V1, added: &["fast"], removed: &[] }],
};

/// The per-element deltas of the attribute-type table.
///
/// One row: rule 21109 of Version 1, "The attribute `fast` on a `Reaction` object must have a value
/// of type `boolean`", which Version 2 removed along with the attribute. The base table has no type
/// for `fast`, so the delta adds one.
pub(crate) static ATTRIBUTE_TYPE_DELTAS: Map<&str, &[TypeDelta]> = phf_map! {
    "reaction" => &[TypeDelta { versions: L3V1, added: &[("fast", "boolean")], removed: &[] }],
};

/// The per-element deltas of the allowed-children table.
///
/// Empty: the two versions of the Level 3 core permit the same child elements. What differs is what
/// they *require* (Version 1 requires a `model`, requires the core lists to be non-empty and
/// requires a `math` child in more places), and a requirement is a rule, not a relaxation - adding
/// it here would turn the table into a rule table. Those rules are enumerated in the report of the
/// unit that landed this module, with their ids (`20201`, `20203`, `20306`, `20804`, `20907`,
/// `21007`, `21101`, `21103`, `21123`, `21130`, `21201`, `21203`, `21209`, `21210`, `21213`,
/// `21231`).
pub(crate) static ALLOWED_CHILD_DELTAS: Map<&str, &[NameDelta]> = phf_map! {};

/// The per-element deltas of the required-children table.
///
/// This is the one table where Version 1 *adds* requirements rather than relaxing them, and the
/// additions are the "exactly one" halves of rules whose Version 2 texts read "may contain at most
/// one". Each row names the rule it comes from:
///
///  * `20306` - a `FunctionDefinition` must contain exactly one `math` element;
///  * `20804` - an `InitialAssignment` must contain exactly one `math` element;
///  * `20907` - every rule (`assignmentRule`, `rateRule`, `algebraicRule`) must contain exactly one
///    `math` element;
///  * `21007` - a `Constraint` must contain exactly one `math` element;
///  * `21130` - a `KineticLaw` must contain exactly one `math` element;
///  * `21209`/`21210`/`21213`/`21231` - a `Trigger`/`Delay`/`EventAssignment`/`Priority` must
///    contain exactly one `math` element;
///  * `21201` - an `Event` must contain exactly one `Trigger`;
///  * `20201` - an SBML document must contain a `Model`.
///
/// A missing required child is a type-check problem, so the element is reported and the walk does
/// not descend into it: that is the same contract every other required child has (see
/// `VALIDATION.md`), and the effect on the report is measured rather than assumed.
pub(crate) static REQUIRED_CHILD_DELTAS: Map<&str, &[NameDelta]> = phf_map! {
    "sbml" => &[NameDelta { versions: L3V1, added: &["model"], removed: &[] }],
    "functionDefinition" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "initialAssignment" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "assignmentRule" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "rateRule" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "algebraicRule" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "constraint" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "kineticLaw" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "trigger" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "delay" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "eventAssignment" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "priority" => &[NameDelta { versions: L3V1, added: &["math"], removed: &[] }],
    "event" => &[NameDelta { versions: L3V1, added: &["trigger"], removed: &[] }],
};

/// The per-element deltas of the unique-children table.
///
/// Empty: the two versions agree on which children may appear at most once.
pub(crate) static UNIQUE_CHILD_DELTAS: Map<&str, &[NameDelta]> = phf_map! {};

/// The deltas of the MathML allowed-children table.
///
/// Rule 10202 lists the MathML subset, and Version 2 added five operators to it (`implies`, `max`,
/// `min`, `quotient` and `rem` are the names the Version 2 specification marks with `\vertwo`), so
/// a Version 1 document that uses one of them has an unknown `<math>` child. This is the row that
/// replaced a hand-maintained copy of the whole 70-name list: the copy differed by exactly these
/// five names and was invisible to the element-name completeness scan, which is the duplication
/// this module exists to prevent.
pub(crate) static MATHML_CHILD_DELTAS: Map<&str, &[NameDelta]> = phf_map! {
    "math" => &[NameDelta {
        versions: L3V1,
        added: &[],
        removed: &["implies", "max", "min", "quotient", "rem"],
    }],
};

/// Apply the delta rows of `deltas` for `element` in `version` to the base row `base`.
///
/// Returns `None` when no delta applies, which is the signal to the callers that they can borrow
/// the base row instead of building one.
fn apply<'a>(
    element: &str,
    version: LevelVersion,
    deltas: &'static Map<&'static str, &'static [NameDelta]>,
    base: &'a [&'static str],
) -> Option<Vec<&'a str>> {
    let rows = deltas.get(element)?;
    let mut result: Option<Vec<&str>> = None;
    for row in rows.iter().filter(|it| it.versions.contains(&version)) {
        let names = result.get_or_insert_with(|| base.to_vec());
        for name in row.added {
            if !names.contains(name) {
                names.push(name);
            }
        }
        names.retain(|it| !row.removed.contains(it));
    }
    result
}

/// The attributes the given element permits in the given version.
///
/// `None` means the base table has no row for the element, which every caller treats as "this
/// element is not one the table describes" (see the callers in
/// [`crate::core::validation::type_check`]).
pub(crate) fn allowed_attributes(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [&'static str]>> {
    let base = *ALLOWED_ATTRIBUTES.get(element)?;
    match apply(element, version, &ALLOWED_ATTRIBUTE_DELTAS, base) {
        Some(names) => Some(Cow::Owned(names)),
        None => Some(Cow::Borrowed(base)),
    }
}

/// The attributes the given element requires in the given version.
pub(crate) fn required_attributes(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [&'static str]>> {
    let base = *REQUIRED_ATTRIBUTES.get(element)?;
    match apply(element, version, &REQUIRED_ATTRIBUTE_DELTAS, base) {
        Some(names) => Some(Cow::Owned(names)),
        None => Some(Cow::Borrowed(base)),
    }
}

/// The typed attributes of the given element in the given version.
pub(crate) fn attribute_types(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [(&'static str, &'static str)]>> {
    let base = *ATTRIBUTE_TYPES.get(element)?;
    let Some(rows) = ATTRIBUTE_TYPE_DELTAS.get(element) else {
        return Some(Cow::Borrowed(base));
    };
    let applicable: Vec<&TypeDelta> = rows
        .iter()
        .filter(|it| it.versions.contains(&version))
        .collect();
    if applicable.is_empty() {
        return Some(Cow::Borrowed(base));
    }
    let mut names = base.to_vec();
    for row in applicable {
        for pair in row.added {
            if !names.contains(pair) {
                names.push(*pair);
            }
        }
        names.retain(|it| !row.removed.contains(&it.0));
    }
    Some(Cow::Owned(names))
}

/// The children the given version *adds* to the requirements of the given element.
///
/// These are the rule-only requirements: a missing one does not make the typed view of the element
/// unusable (the crate reads them as optional children), so they are reported by the walk and the
/// object is still validated - see [`base_required_children`] for the other half.
pub(crate) fn added_required_children(
    element: &str,
    version: LevelVersion,
) -> &'static [&'static str] {
    let Some(rows) = REQUIRED_CHILD_DELTAS.get(element) else {
        return &[];
    };
    let Some(row) = rows.iter().find(|it| it.versions.contains(&version)) else {
        return &[];
    };
    row.added
}

/// The children the given element permits in the given version.
pub(crate) fn allowed_children(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [&'static str]>> {
    let base = *ALLOWED_CHILDREN.get(element)?;
    match apply(element, version, &ALLOWED_CHILD_DELTAS, base) {
        Some(names) => Some(Cow::Owned(names)),
        None => Some(Cow::Borrowed(base)),
    }
}

/// The children the *base* table requires of the given element, without any version delta.
///
/// This is the half of `required_children` that the type check uses as a guard: a child the base
/// table requires is one the typed view of the element reads as a required child, so a missing one
/// makes the view unusable. The requirements a *version* adds on top (Version 1's "exactly one
/// `math`" rules) are not like that - the crate's `math` accessors are optional children - so they
/// are reported by the walk instead, and this function is what keeps the two apart.
pub(crate) fn base_required_children(element: &str) -> &'static [&'static str] {
    REQUIRED_CHILDREN.get(element).copied().unwrap_or(&[])
}

/// The children the given element requires in the given version.
///
/// The composition of the two halves the validation layer keeps apart, and used by the tests that
/// pin them against each other:
/// [`base_required_children`] (a child the typed view reads as required, so a missing one stops the
/// walk of the element) and [`added_required_children`] (a requirement the document's own version
/// adds, reported as a rule without stopping the walk).
#[cfg(test)]
pub(crate) fn required_children(
    element: &str,
    version: LevelVersion,
) -> Cow<'static, [&'static str]> {
    let base = base_required_children(element);
    let added = added_required_children(element, version);
    if added.is_empty() {
        return Cow::Borrowed(base);
    }
    let mut names = base.to_vec();
    for name in added {
        if !names.contains(name) {
            names.push(name);
        }
    }
    Cow::Owned(names)
}

/// The children that may appear at most once in the given element, in the given version.
pub(crate) fn unique_children(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [&'static str]>> {
    let base = *UNIQUE_CHILDREN.get(element)?;
    match apply(element, version, &UNIQUE_CHILD_DELTAS, base) {
        Some(names) => Some(Cow::Owned(names)),
        None => Some(Cow::Borrowed(base)),
    }
}

/// The MathML elements the given MathML element permits in the given version.
pub(crate) fn mathml_children(
    element: &str,
    version: LevelVersion,
) -> Option<Cow<'static, [&'static str]>> {
    let base = *MATHML_ALLOWED_CHILDREN.get(element)?;
    match apply(element, version, &MATHML_CHILD_DELTAS, base) {
        Some(names) => Some(Cow::Owned(names)),
        None => Some(Cow::Borrowed(base)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::element::{
        ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES, MATHML_ALLOWED_CHILDREN,
        REQUIRED_ATTRIBUTES, REQUIRED_CHILDREN, UNIQUE_CHILDREN,
    };

    /// Every delta row of the module, as `(table, element, versions, added, removed)`.
    ///
    /// The rows are collected from the tables themselves rather than written down again, so a row
    /// that is added without being covered by the consistency rules below is impossible: the scan
    /// reads the same data the accessors do.
    fn all_rows() -> Vec<(
        &'static str,
        &'static str,
        &'static [LevelVersion],
        &'static [&'static str],
        &'static [&'static str],
    )> {
        let mut rows = Vec::new();
        for (table, entries) in [
            (
                "ALLOWED_ATTRIBUTES",
                ALLOWED_ATTRIBUTE_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
            (
                "REQUIRED_ATTRIBUTES",
                REQUIRED_ATTRIBUTE_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
            (
                "ALLOWED_CHILDREN",
                ALLOWED_CHILD_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
            (
                "REQUIRED_CHILDREN",
                REQUIRED_CHILD_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
            (
                "UNIQUE_CHILDREN",
                UNIQUE_CHILD_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
            (
                "MATHML_ALLOWED_CHILDREN",
                MATHML_CHILD_DELTAS
                    .entries()
                    .map(|(key, value)| (*key, value))
                    .collect::<Vec<_>>(),
            ),
        ] {
            for (element, deltas) in entries {
                for delta in deltas.iter() {
                    rows.push((table, element, delta.versions, delta.added, delta.removed));
                }
            }
        }
        rows
    }

    /// The base row a delta row claims to change, or an empty row when the table has none.
    ///
    /// A missing row is only legitimate for `REQUIRED_CHILDREN`, where a delta may *introduce* a
    /// requirement the base table does not have (Version 1's "exactly one" halves); the assertion
    /// below pins that, so a delta naming an element no other table describes still fails.
    fn base_row(table: &str, element: &str) -> Vec<&'static str> {
        if table == "REQUIRED_CHILDREN" && REQUIRED_CHILDREN.get(element).is_none() {
            return Vec::new();
        }
        match table {
            "ALLOWED_ATTRIBUTES" => ALLOWED_ATTRIBUTES.get(element).map(|it| it.to_vec()),
            "REQUIRED_ATTRIBUTES" => REQUIRED_ATTRIBUTES.get(element).map(|it| it.to_vec()),
            "ALLOWED_CHILDREN" => ALLOWED_CHILDREN.get(element).map(|it| it.to_vec()),
            "REQUIRED_CHILDREN" => REQUIRED_CHILDREN.get(element).map(|it| it.to_vec()),
            "UNIQUE_CHILDREN" => UNIQUE_CHILDREN.get(element).map(|it| it.to_vec()),
            "MATHML_ALLOWED_CHILDREN" => MATHML_ALLOWED_CHILDREN.get(element).map(|it| it.to_vec()),
            other => panic!("the scan does not know the table {other}"),
        }
        .unwrap_or_else(|| {
            panic!(
                "the delta of `{table}` names `{element}`, which the base table does not describe"
            )
        })
    }

    /// A delta row is a statement about a base row, so it has to be *consistent* with it: it names
    /// an element the base table describes, it removes names the base row has, it adds names the
    /// base row does not have, and it changes something.
    ///
    /// Without this, a delta row is a silent no-op - the failure mode that makes hand-maintained
    /// copies of a table drift - because nothing would compare it against the row it claims to
    /// describe. The test reads the same tables the accessors read, so it cannot go stale.
    /// The one table where a version *adds* requirements: Version 1 requires the `math` child (and
    /// the `trigger` of an `event`, and the `model` of a document) that Version 2 makes optional, and
    /// the two halves the validation layer reads are kept apart on purpose - the base row is what
    /// stops the walk of an element, the added names are a rule that does not.
    #[test]
    fn the_version_1_requirements_are_added_and_the_version_2_ones_are_not() {
        for (element, added) in [
            ("trigger", "math"),
            ("delay", "math"),
            ("eventAssignment", "math"),
            ("priority", "math"),
            ("functionDefinition", "math"),
            ("initialAssignment", "math"),
            ("assignmentRule", "math"),
            ("rateRule", "math"),
            ("algebraicRule", "math"),
            ("constraint", "math"),
            ("kineticLaw", "math"),
            ("event", "trigger"),
            ("sbml", "model"),
        ] {
            assert!(
                !required_children(element, LevelVersion::L3V1).is_empty(),
                "{element} requires nothing in Version 1"
            );
            assert!(
                required_children(element, LevelVersion::L3V1).contains(&added),
                "{element} does not require `{added}` in Version 1"
            );
            assert!(
                !required_children(element, LevelVersion::L3V2).contains(&added),
                "{element} requires `{added}` in Version 2, which the specification does not"
            );
            assert!(
                added_required_children(element, LevelVersion::L3V1).contains(&added)
                    && added_required_children(element, LevelVersion::L3V2).is_empty(),
                "{element} does not keep the Version 1 addition out of the walk guard"
            );
            assert!(
                !base_required_children(element).contains(&added),
                "{element} reads `{added}` as a required child of the typed view"
            );
        }
        // The other direction: a child the base table requires is required in both versions.
        for element in ["boundingBox", "curve", "listOfLayouts"] {
            assert_eq!(
                base_required_children(element),
                required_children(element, LevelVersion::L3V2).as_ref(),
                "{element} lost its base requirement"
            );
        }
    }

    #[test]
    fn every_delta_row_is_consistent_with_the_base_row_it_names() {
        let mut rows = 0usize;
        let mut changes = 0usize;
        for (table, element, versions, added, removed) in all_rows() {
            rows += 1;
            assert!(
                !versions.is_empty(),
                "the delta of `{table}`/`{element}` applies to no version, so it is unreachable"
            );
            let base = base_row(table, element);
            for name in removed {
                assert!(
                    base.contains(name),
                    "the delta of `{table}`/`{element}` removes `{name}`, which the base row does \
                     not have"
                );
            }
            for name in added {
                assert!(
                    !base.contains(name),
                    "the delta of `{table}`/`{element}` adds `{name}`, which the base row already \
                     has"
                );
            }
            changes += usize::from(!added.is_empty() || !removed.is_empty());
        }
        // A guard against a scan that silently reads nothing: the module has rows, and every one of
        // them changes its row.
        assert!(rows >= 30, "the scan found only {rows} delta rows");
        assert_eq!(
            rows,
            changes,
            "{} delta rows change nothing, so they are unreachable",
            rows - changes
        );
    }

    /// The attribute-type deltas are rows of a different shape (pairs rather than names), so they
    /// are checked the same way: the element is described by the base table, an added pair is not
    /// there yet, and a removed attribute is.
    #[test]
    fn every_attribute_type_delta_row_is_consistent_with_its_base_row() {
        let mut rows = 0usize;
        for (element, deltas) in ATTRIBUTE_TYPE_DELTAS.entries() {
            let base = ATTRIBUTE_TYPES.get(*element).unwrap_or_else(|| {
                panic!("the type delta names `{element}`, which the base table does not describe")
            });
            for delta in deltas.iter() {
                rows += 1;
                assert!(!delta.versions.is_empty());
                for pair in delta.added {
                    assert!(
                        !base.contains(pair),
                        "the type delta of `{element}` adds `{pair:?}`, which the base row has"
                    );
                }
                for name in delta.removed {
                    assert!(
                        base.iter().any(|it| it.0 == *name),
                        "the type delta of `{element}` removes `{name}`, which the base row lacks"
                    );
                }
                assert!(!delta.added.is_empty() || !delta.removed.is_empty());
            }
        }
        assert!(rows >= 1, "the scan found no attribute-type delta rows");
    }

    /// The headline delta of the unit, pinned in both directions: `fast` is an attribute of a
    /// reaction in Level 3 Version 1 (rules 21110 and 21109 of Version 1) and is not one in
    /// Version 2, where the specification removes it.
    #[test]
    fn fast_is_a_version_1_reaction_attribute_and_not_a_version_2_one() {
        for version in [LevelVersion::L3V1, LevelVersion::L3V2] {
            let allowed = allowed_attributes("reaction", version).expect("the table has a row");
            let required = required_attributes("reaction", version).expect("the table has a row");
            let types = attribute_types("reaction", version).expect("the table has a row");
            let is_v1 = version == LevelVersion::L3V1;
            assert_eq!(
                allowed.contains(&"fast"),
                is_v1,
                "`fast` must be allowed exactly in Version 1 ({version:?})"
            );
            assert_eq!(
                required.contains(&"fast"),
                is_v1,
                "`fast` must be required exactly in Version 1 ({version:?})"
            );
            assert_eq!(
                types.iter().any(|it| *it == ("fast", "boolean")),
                is_v1,
                "`fast` must be typed exactly in Version 1 ({version:?})"
            );
        }
    }

    /// The other direction of the delta: Version 2 added the optional `id` and `name` attributes to
    /// a set of elements, so Version 1 does not permit them.
    ///
    /// The elements are the ones the per-rule texts name (20108, 20223-20232, 20420, 20421, 20805,
    /// 20908-20910, 21009, 21129, 21132, 21150, 21151, 21214, 21224, 21226, 21227, 21232); the
    /// test reads the delta table, so it cannot disagree with what the accessors do.
    #[test]
    fn the_attributes_version_2_added_are_absent_in_version_1() {
        let mut elements = 0usize;
        for (element, deltas) in ALLOWED_ATTRIBUTE_DELTAS.entries() {
            if !deltas.iter().any(|it| it.removed == ID_AND_NAME) {
                continue;
            }
            elements += 1;
            let v1 = allowed_attributes(element, LevelVersion::L3V1).expect("the table has a row");
            let v2 = allowed_attributes(element, LevelVersion::L3V2).expect("the table has a row");
            for name in ID_AND_NAME {
                assert!(
                    !v1.contains(name),
                    "`{element}@{name}` is not permitted in Version 1"
                );
                assert!(
                    v2.contains(name),
                    "`{element}@{name}` is permitted in Version 2"
                );
            }
        }
        // The list is not written down twice: the count is pinned, so a row that stops removing
        // `id`/`name` fails here rather than silently widening Version 1.
        assert_eq!(
            elements,
            SBASE_ATTRIBUTES_WITHOUT_ID_AND_NAME.len(),
            "every element of the list has to have its removal row"
        );
    }

    /// The MathML subset of rule 10202 is a delta row now: the five operators Version 2 added are
    /// not in the Version 1 subset, and every other name is unchanged.
    ///
    /// This replaced a hand-maintained copy of the whole 70-name list, which differed by exactly
    /// these five names and which the element-name completeness scan never read.
    #[test]
    fn the_mathml_subset_delta_is_exactly_the_five_operators_version_2_added() {
        const ADDED_IN_VERSION_2: &[&str] = &["implies", "max", "min", "quotient", "rem"];
        let v1 = mathml_children("math", LevelVersion::L3V1).expect("the table has a row");
        let v2 = mathml_children("math", LevelVersion::L3V2).expect("the table has a row");
        let base = MATHML_ALLOWED_CHILDREN["math"];

        assert_eq!(v2.as_ref(), base, "Version 2 is the base table");
        assert_eq!(
            v1.len(),
            base.len() - ADDED_IN_VERSION_2.len(),
            "the Version 1 subset is the Version 2 one without the five operators"
        );
        for name in ADDED_IN_VERSION_2 {
            assert!(v2.contains(name), "`{name}` is in the Version 2 subset");
            assert!(
                !v1.contains(name),
                "`{name}` is not in the Version 1 subset"
            );
        }
        for name in base {
            if !ADDED_IN_VERSION_2.contains(name) {
                assert!(v1.contains(name), "`{name}` is in both subsets");
            }
        }
    }

    /// A row no delta mentions borrows the base table, which is what keeps the common case
    /// allocation-free.
    #[test]
    fn a_row_without_a_delta_borrows_the_base_table() {
        let base = *ALLOWED_ATTRIBUTES
            .get("species")
            .expect("the table has a row");
        let resolved =
            allowed_attributes("species", LevelVersion::L3V1).expect("the table has a row");
        assert!(matches!(resolved, Cow::Borrowed(_)));
        assert_eq!(resolved.as_ref(), base);
    }
}
