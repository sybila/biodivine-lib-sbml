use std::collections::HashSet;

use const_format::formatcp;
use regex::Regex;

use crate::constants::element::{ALLOWED_CHILDREN, MATHML_ALLOWED_CHILDREN};
use crate::core::{BaseUnit, Model, SBase};
use crate::xml::OptionalXmlProperty;
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
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    );
}

/// Executes a validation of xml list object itself and all its children.
pub(crate) fn validate_list_of_objects<T: SbmlValidable>(
    list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<String>,
    meta_ids: &mut HashSet<String>,
) {
    let allowed = get_allowed_children(list.xml_element());
    let xml_element = list.xml_element();
    let id = list.id();
    let meta_id = list.meta_id();

    apply_rule_10301(id.get(), xml_element, issues, identifiers);
    apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
    apply_rule_10308(list.sbo_term().get(), xml_element, issues);
    apply_rule_10309(meta_id.get(), xml_element, issues);
    apply_rule_10310(id.get(), xml_element, issues);
    apply_rule_10312(list.name().get(), xml_element, issues);

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
fn check_identifier_uniqueness(
    rule: &str,
    attr_name: &str,
    identifier: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<String>,
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
fn matches_sid_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **SBOTerm** syntax.
fn matches_sboterm_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^SBO:\d{7}$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **XML 1.0 ID** syntax.
fn matches_xml_id_pattern(value: &Option<String>) -> bool {
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

// TODO: Complete implementation when adding extension/packages is solved
/// ### Rule 10301
/// The value of the attribute id on every instance of the following classes of objects must be unique
/// across the set of all id attribute values of all such objects in a model:
/// [AlgebraicRule](crate::core::rule::AlgebraicRule), [AssignmentRule](crate::core::rule::AssignmentRule),
/// [Compartment](compartment::Compartment), [Constraint](constraint::Constraint), [Delay](event::Delay),
/// [Event](event::Event), [EventAssignment](event::EventAssignment),
/// [FunctionDefinition](function_definition::FunctionDefinition),
/// [InitialAssignment](initial_assignment::InitialAssignment), [KineticLaw](reaction::KineticLaw),
/// [ListOfCompartments](Model::compartments), [ListOfConstraints](Model::constraints),
/// [ListOfEventAssignments](event::Event::event_assignments), [ListOfEvents](Model::events),
/// [ListOfFunctionDefinitions](Model::function_definitions),
/// [ListOfInitialAssignments](Model::initial_assignments),
/// [ListOfLocalParameters](reaction::KineticLaw::local_parameters),
/// [ListOfModifierSpeciesReferences](reaction::Reaction::modifiers), [ListOfParameters](Model::parameters),
/// [ListOfReactions](Model::reactions), [ListOfRules](Model::rules),
/// [ListOfSpecies](Model::species), [ListOfSpeciesReferences](reaction::Reaction::reactants),
/// [ListOfUnitDefinitions](Model::unit_definitions), [ListOfUnits](unit_definition::UnitDefinition::units),
/// [Model](Model), [ModifierSpeciesReference](reaction::ModifierSpeciesReference),
/// [Parameter](parameter::Parameter), [Priority](event::Priority), [RateRule](rule::RateRule),
/// [Reaction](reaction::Reaction), [Species](species::Species), [SpeciesReference](reaction::SpeciesReference),
/// [Trigger](event::Trigger), and [Unit](unit::Unit), plus the *id* attribute values of any SBML Level 3 package
/// element defined to be in the *SId* namespace of the [Model](Model).
pub(crate) fn apply_rule_10301(
    id: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<String>,
) {
    check_identifier_uniqueness("10301", "id", id, xml_element, issues, identifiers);
}

/// ### Rule 10307
/// Every *metaid* attribute value must be unique across the set of all *metaid* values in a model.
// TODO: might be placed inside SBASE validation method
pub(crate) fn apply_rule_10307(
    meta_id: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    meta_ids: &mut HashSet<String>,
) {
    check_identifier_uniqueness("10307", "meta_id", meta_id, xml_element, issues, meta_ids);
}

// TODO: might be placed inside SBASE validation method
/// ### Rule 10308
/// The value of the attribute *sboTerm* must always conform to the syntax of the SBML data type
/// **SBOTerm**, which is a string consisting of the characters `S', `B', `O', ':', followed by
/// exactly seven digits.
pub(crate) fn apply_rule_10308(
    sbo_term: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_sboterm_pattern(&sbo_term) {
        let message = format!(
            "The [sboTerm] attribute value ('{0}') does not conform to the syntax of SBOTerm data type.",
            sbo_term.unwrap()
        );
        issues.push(SbmlIssue::new_error("10308", xml_element, message))
    }
}

// TODO: might be placed inside SBASE validation method
/// ### Rule 10309
/// The value of a *metaid* attribute must always conform to the syntax of the *XML* data type **ID**.
pub(crate) fn apply_rule_10309(
    meta_id: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_xml_id_pattern(&meta_id) {
        let message = format!(
            "The [metaId] attribute value ('{0}') does not conform to the syntax of XML 1.0 ID data type.",
            meta_id.unwrap()
        );
        issues.push(SbmlIssue::new_error("10309", xml_element, message))
    }
}

/// ### Rule 10310
/// The value of an *id* attribute must always conform to the syntax of the SBML data type **SId**.
pub(crate) fn apply_rule_10310(
    id: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_sid_pattern(&id) {
        let message = format!(
            "The [id] attribute value ('{0}') does not conform to the syntax of SId data type.",
            id.unwrap()
        );
        issues.push(SbmlIssue::new_error("10310", xml_element, message))
    }
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

// TODO: might be placed inside SBASE validation method
/// ### Rule 10312
/// The value of a **name** attribute must always conform to the syntax of type **string**.
pub(crate) fn apply_rule_10312(
    name: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_xml_string_pattern(&name) {
        let message = format!(
            "The [name] attribute value ('{0}') does not conform to the syntax of XML 1.0 string data type.",
            name.unwrap()
        );
        issues.push(SbmlIssue::new_error("10312", xml_element, message))
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
    unit_ref: Option<String>,
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

// TODO: might be placed inside SBASE validation method
/// ### Rule 10401
/// Every top-level XML element within an **Annotation** object must have an XML namespace declared.
pub(crate) fn apply_rule_10401(annotation: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let top_level_elements = annotation.child_elements();

    for element in top_level_elements {
        // TODO: is this correct and sufficient?
        if element.namespace_url().is_empty() {
            let message = format!(
                "XML namespace not declared for '{0}' in annotation.",
                element.full_name()
            );
            issues.push(SbmlIssue::new_error(
                "10401",
                element.xml_element(),
                message,
            ))
        }
    }
}

// TODO: might be placed inside SBASE validation method
/// ### Rule 10402
/// A given XML namespace cannot be the namespace of more than *one* top-level element within a
// given **Annotation** object.
pub(crate) fn apply_rule_10402(annotation: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let top_level_elements =
        annotation.child_elements_filtered(|el| !el.namespace_url().is_empty());
    let mut unique_namespaces: HashSet<String> = HashSet::new();

    for element in top_level_elements {
        let namespace = element.namespace_url();

        if unique_namespaces.contains(&namespace) {
            let message = format!(
                "XML namespace '{namespace}' found in multiple top-level elements of <annotation>."
            );
            issues.push(SbmlIssue::new_error("10402", &element, message));
        } else {
            unique_namespaces.insert(namespace);
        }
    }
}
