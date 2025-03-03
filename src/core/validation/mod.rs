use const_format::formatcp;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use crate::constants::element::{ALLOWED_CHILDREN, MATHML_ALLOWED_CHILDREN};
use crate::core::sbase::SId;
use crate::core::validation::sbase::validate_sbase;
use crate::core::{BaseUnit, MetaId, Model};
use crate::xml::XmlElement;
use crate::xml::XmlList;
use crate::xml::XmlWrapper;
use crate::SbmlIssue;

mod compartment;
mod constraint;
mod event;
mod function_definition;
mod initial_assignment;
mod math;
mod model;
mod parameter;
mod reaction;
mod rule;
pub(crate) mod sbase;
mod species;
/// This module implements basic integrity checks that are necessary to enable full validation.
/// In general, these should ensure that our XML abstractions work as expected and the model
/// is safe to work with. In particular:
///  - All required properties are set.
///  - All properties have the correct type.
///  - All required child elements are present.
///  - All child elements appear only once in their parents.
///  - All XML lists only contain children of the allowed type.
///  - Namespaces are correctly applied.
///
/// Most of these checks are technically covered by rule 10102, but for most SBML elements, a more
/// specific rule ID exists as well. In order to avoid implementing all these rules
/// independently, we perform a single recursive "type check" procedure that then maps
/// any discovered issues to the correct rule ID (and defaults to 10102 or other appropriate rule
/// when an element-specific rule does not exist).
pub(crate) mod type_check;
mod unit;
mod unit_definition;
mod xml_definitions;

/// Denotes an element that can be (and should be) validated against the SBML
/// validation rules.
pub(crate) trait SbmlValidable: XmlWrapper {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    );
}

/// Executes a validation of xml list object itself and all its children.
pub(crate) fn validate_list_of_objects<T: SbmlValidable>(
    list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<SId>,
    meta_ids: &mut HashSet<MetaId>,
) {
    validate_sbase(list, issues, identifiers, meta_ids);

    let allowed = get_allowed_children(list.xml_element());
    for object in list.as_vec() {
        if allowed.contains(&object.tag_name().as_str()) {
            object.validate(issues, identifiers, meta_ids);
        }
    }
}

pub(crate) fn get_allowed_children(xml_element: &XmlElement) -> &'static [&'static str] {
    let tag_name = xml_element.tag_name();
    if let Some(allowed) = ALLOWED_CHILDREN.get(&tag_name) {
        allowed
    } else if let Some(allowed) = MATHML_ALLOWED_CHILDREN.get(&tag_name) {
        allowed
    } else {
        &[]
    }
}

/// Checks that a given identifier is unique in the given set of identifiers. If the identifier
/// is unique, it is included in the given set of identifiers, otherwise error is logged in the
/// vector of issues.
fn check_identifier_uniqueness<ID: Eq + Hash + Display>(
    rule: &str,
    attr_name: &str,
    identifier: Option<ID>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<ID>,
) {
    if let Some(identifier) = identifier {
        if identifiers.contains(&identifier) {
            let tag_name = xml_element.tag_name();
            let message = format!(
                "The {attr_name} ('{identifier}') of <{tag_name}> is already present in the <model>."
            );
            issues.push(SbmlIssue::new_error(rule, xml_element, message));
        } else {
            identifiers.insert(identifier);
        }
    }
}

/// Checks that a given value conforms to the syntax described in pattern.
fn matches_pattern(value: &Option<String>, pattern: &Regex) -> bool {
    if let Some(value) = value {
        return pattern.is_match(value);
    }
    true
}

/// Check that a given value conforms to the **SId** syntax.
pub(crate) fn matches_sid_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **SBOTerm** syntax.
pub(crate) fn matches_sboterm_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^SBO:\d{7}$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **XML 1.0 ID** syntax.
pub(crate) fn matches_xml_id_pattern(value: &Option<String>) -> bool {
    let pattern = formatcp!(
        "^[{0}_:][{0}{1}.\\-_:{2}{3}]*$",
        xml_definitions::build_letter_group(),
        xml_definitions::build_digit_group(),
        xml_definitions::build_combining_char_group(),
        xml_definitions::build_extender_group(),
    );
    let pattern = Regex::new(pattern).unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conform to the **UnitSId** syntax, which is the same as **SId** syntax.
fn matches_unit_sid_pattern(value: &Option<String>) -> bool {
    matches_sid_pattern(value)
}

fn matches_xml_string_pattern(value: &Option<String>) -> bool {
    // TODO:
    //      The `&` `'` and `"` escaping is probably handled by `xml-doc` and we should just see
    //      "normal", unescaped strings in XML attributes, hence this check is probably a bit
    //      too aggressive. But we should make sure to test this.
    let pattern =
        Regex::new(r###"^[^&'"\uFFFE\uFFFF]*(?:&(amp|apos|quot);[^&'"\uFFFE\uFFFF]*)*$"###)
            .unwrap();
    matches_pattern(value, &pattern)
}

/// ### Rule 10311
/// Unit identifiers (that is, the values of the **id** attribute on
/// [UnitDefinition](unit_definition::UnitDefinition), the **units** attribute
/// on [Compartment](compartment::Compartment), the **units** attribute on
/// [Parameter](parameter::Parameter), the **units** attribute on
/// [LocalParameter](reaction::LocalParameter), the **substanceUnits** attribute on
/// [Species](species::Species), the SBML **units** attribute on MathML **cn** elements, and the
/// **substanceUnits**, **volumeUnits**, **areaUnits**, **lengthUnits**, **timeUnits** and
/// **extentUnits** on [Model]) must always conform to the syntax of the SBML
/// data type **UnitSId**.
pub(crate) fn apply_rule_10311(
    attr_name: &str,
    unit_sid: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_unit_sid_pattern(&unit_sid) {
        let message = format!(
            "The [{attr_name}] attribute value ('{0}') does not conform to the syntax of UnitSId data type.",
            unit_sid.unwrap()
        );
        issues.push(SbmlIssue::new_error("10311", xml_element, message))
    }
}

/// ### Rule 10313
/// Unit identifier references (that is, the *units* attribute on
/// [Compartment](compartment::Compartment), the *units* attribute on
/// [Parameter](parameter::Parameter), the *units* attribute on
/// [LocalParameter](reaction::LocalParameter), the *substanceUnits* attribute on [Species],
/// the SBML *units* attribute on MathML **ci** elements, and the *substanceUnits*, *volumeUnits*,
/// *areaUnits*, *lengthUnits*, *timeUnits* and *extentUnits* on [Model]) must be the identifier of
/// a [UnitDefinition] in the [Model], or the identifier of a predefined unit in SBML, that is, any
/// of the following [BaseUnit]: `ampere`, `avogadro`, `becquerel`, `candela`, `coulomb`, `dimensionless`, `farad`, `gram`,
// `gray`, `henry`, `hertz`, `item`, `joule`, `katal`, `kelvin`, `kilogram`, `litre`, `lumen`,
// `lux`, `metre`, `mole`, `newton`, `ohm`, `pascal`, `radian`, `second`, `siemens`, `sievert`,
// `steradian`, `tesla`, `volt`, `watt`, or `weber`.
pub(crate) fn apply_rule_10313(
    attr_name: &str,
    unit_ref: Option<SId>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    let Some(unit_ref) = unit_ref else {
        return;
    };
    // TODO: could be optimized - make efficient passing of the vector of unit definition identifiers or use global variable or something else
    let unit_definition_ids = Model::for_child_element(xml_element)
        .unwrap()
        .unit_definition_identifiers();

    if !unit_definition_ids.contains(&unit_ref) && BaseUnit::try_from(unit_ref.as_str()).is_err() {
        let message = format!(
            "The [{attr_name}] attribute value ('{unit_ref}') is not a \
        known <unitDefinition> identifier nor a valid base unit."
        );
        issues.push(SbmlIssue::new_error("10313", xml_element, message));
    }
}
