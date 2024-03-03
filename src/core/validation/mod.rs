use crate::constants::element::{
    ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES, MATHML_ALLOWED_CHILDREN,
    REQUIRED_ATTRIBUTES,
};
use crate::constants::namespaces::URL_SBML_CORE;
use crate::core::{BaseUnit, Model, SBase};
use crate::xml::{
    DynamicProperty, OptionalXmlProperty, XmlElement, XmlList, XmlProperty, XmlPropertyType,
    XmlWrapper,
};
use crate::{Sbml, SbmlIssue};
use regex::Regex;
use std::collections::HashSet;
use std::ops::Deref;
use xml_doc::Element;

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
#[cfg(test)]
mod test_suite;
mod unit;
mod unit_definition;

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

/// Denotes an element that possess a way to self-test against
/// the most critical checks (sanity test). This should be executed **before** actual document
/// validation. Failing sanity tests skips the validation. That is, because reading such a (insane)
/// document would cause panic.
pub(crate) trait SanityCheckable: XmlWrapper {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);
    }
}

impl Sbml {
    /// ### Rule 10102
    /// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
    /// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
    /// elements or attributes placed in an SBML namespace do not conform to the SBML
    /// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
    pub(crate) fn apply_rule_10102(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.xml.read().unwrap();

        if doc.container().child_elements(doc.deref()).len() != 1 {
            let container = XmlElement::new_raw(self.xml.clone(), doc.container());
            let message = "The document contains multiple root nodes. \
                Only one root <sbml> object is allowed.";
            issues.push(SbmlIssue::new_error("10102", &container, message));
        }

        let root_element = self.sbml_root.xml_element();
        if root_element.tag_name() == "sbml" {
            validate_allowed_attributes(
                root_element,
                &root_element
                    .attributes()
                    .keys()
                    .map(|key| key.as_str())
                    .collect::<Vec<&str>>(),
                issues,
            );

            validate_allowed_children(
                root_element,
                &root_element
                    .child_elements()
                    .iter()
                    .map(|xml_element| xml_element.raw_element().full_name(doc.deref()))
                    .collect(),
                issues,
            );
        } else {
            let message = format!("Invalid root element <{}> found.", root_element.tag_name());
            issues.push(SbmlIssue::new_error("10102", &self.sbml_root, message));
        }
    }
}

/// Performs very basic and the most critical sanity checks. more precisely:
/// - the document contains all required children and attributes.
/// - each attribute value has correct type.
/// Any failing check is logged in *issues*.
pub(crate) fn sanity_check(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let attributes = xml_element.attributes();
    let element_name = xml_element.tag_name();

    if let Some(required) = REQUIRED_ATTRIBUTES.get(element_name.as_str()) {
        for req_attr in required.iter() {
            if !attributes.contains_key(&req_attr.to_string()) {
                // TODO:
                //      These have their own SBML issue IDs assigned to them, and we should
                //      probably try to use them here as well.
                let message = format!(
                    "Sanity check failed: missing required attribute [{req_attr}] on <{element_name}>."
                );
                issues.push(SbmlIssue::new_error("SANITY_CHECK", xml_element, message));
            }
        }
    }

    // check that each attribute contains a value of the correct type
    for attr in attributes {
        let attr_name = attr.0.as_str();
        let Some(types) = ATTRIBUTE_TYPES.get(element_name.as_str()) else {
            break;
        };

        // t => (attribute name, attribute value)
        for t in types {
            if &attr_name == t.0 {
                match *t.1 {
                    "positive_int" => sanity_type_check::<u32>(attr_name, xml_element, issues),
                    "int" => sanity_type_check::<i32>(attr_name, xml_element, issues),
                    "double" => sanity_type_check::<f64>(attr_name, xml_element, issues),
                    "boolean" => sanity_type_check::<bool>(attr_name, xml_element, issues),
                    _ => (),
                }
            };
        }
    }
}

/// Performs a type check of a value of a specific attribute.
/// If check fails, error is logged in *issues*.
fn sanity_type_check<T: XmlPropertyType>(
    attribute_name: &str,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    let property = DynamicProperty::<T>::new(xml_element, attribute_name).get_checked();
    if property.is_err() {
        let message = format!(
            "Sanity check failed: {0} On the attribute [{1}].",
            property.err().unwrap(),
            attribute_name
        );
        issues.push(SbmlIssue::new_error("SANITY_CHECK", xml_element, message));
    }
}

pub(crate) fn sanity_check_of_list<T: SanityCheckable>(
    xml_list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
) {
    sanity_check(xml_list.xml_element(), issues);

    for object in xml_list.iter() {
        object.sanity_check(issues);
    }
}

/// Validates for a given element that its attributes (keys) are only from predefined set of
/// attributes (keys). If not, an error is logged in the vector of issues.
pub(crate) fn validate_allowed_attributes(
    xml_element: &XmlElement,
    attributes: &Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let element_name = xml_element.tag_name();
    let allowed_attributes = ALLOWED_ATTRIBUTES.get(element_name.as_str()).unwrap();

    for full_name in attributes {
        let (_prefix, attr_name) = Element::separate_prefix_name(full_name);
        if !allowed_attributes.contains(&attr_name) {
            let message = format!(
                "An unknown attribute [{}] of the element <{}> found.",
                attr_name, element_name
            );
            issues.push(SbmlIssue::new_error("10102", xml_element, message));
        }
    }
}

/// Validates for a given element that its children (tag names) are only from predefined set of
/// children (tag names). If not, an error is logged in the vector of issues.
pub(crate) fn validate_allowed_children(
    xml_element: &XmlElement,
    children_names: &Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let element_name = xml_element.tag_name();
    let allowed_children = ALLOWED_CHILDREN.get(element_name.as_str()).unwrap();

    for child_full_name in children_names {
        let (_prefix, child_name) = Element::separate_prefix_name(child_full_name);
        if !allowed_children.contains(&child_name) {
            let message = format!(
                "An unknown child <{}> of the element <{}> found.",
                child_name, element_name
            );
            issues.push(SbmlIssue::new_error("10102", xml_element, message));
        }
    }
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

    apply_rule_10102(list.xml_element(), issues);
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
    let pattern = Regex::new(r"^([a-zA-Z]|_)([a-zA-Z]|/d|_)*").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **SBOTerm** syntax.
fn matches_sboterm_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"SBO:\d{7}").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **XML 1.0 ID** syntax.
fn matches_xml_id_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^(\p{L}|_|:)(\p{L}|\d|\.|-|_|:|\p{M})*").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conform to the **UnitSId** syntax, which is the same as **SId** syntax.
fn matches_unit_sid_pattern(value: &Option<String>) -> bool {
    matches_sid_pattern(value)
}

fn matches_xml_string_pattern(value: &Option<String>) -> bool {
    let pattern =
        Regex::new(r###"^[^&'"\uFFFE\uFFFF]*(?:&(amp|apos|quot);[^&'"\uFFFE\uFFFF]*)*$"###)
            .unwrap();
    matches_pattern(value, &pattern)
}

/// ### Rule 10102
/// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
/// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
/// elements or attributes placed in an SBML namespace do not conform to the SBML
/// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
pub(crate) fn apply_rule_10102(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let doc = xml_element.read_doc();
    let element = xml_element.raw_element();
    let attributes = element
        .attributes(doc.deref())
        .keys()
        .map(|key| key.as_str())
        .collect::<Vec<&str>>();
    let children_names = element
        .child_elements(doc.deref())
        .iter()
        .filter(|element| element.namespace(doc.deref()) == Some(URL_SBML_CORE))
        .map(|element| element.full_name(doc.deref()))
        .collect();

    validate_allowed_attributes(xml_element, &attributes, issues);
    validate_allowed_children(xml_element, &children_names, issues);
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
/// [ListOfCompartments](model::Model::compartments), [ListOfConstraints](model::Model::constraints),
/// [ListOfEventAssignments](event::Event::event_assignments), [ListOfEvents](model::Model::events),
/// [ListOfFunctionDefinitions](model::Model::function_definitions),
/// [ListOfInitialAssignments](model::Model::initial_assignments),
/// [ListOfLocalParameters](reaction::KineticLaw::local_parameters),
/// [ListOfModifierSpeciesReferences](reaction::Reaction::modifiers), [ListOfParameters](model::Model::parameters),
/// [ListOfReactions](model::Model::reactions), [ListOfRules](model::Model::rules),
/// [ListOfSpecies](model::Model::species), [ListOfSpeciesReferences](reaction::Reaction::reactants),
/// [ListOfUnitDefinitions](model::Model::unit_definitions), [ListOfUnits](unit_definition::UnitDefinition::units),
/// [Model](model::Model), [ModifierSpeciesReference](reaction::ModifierSpeciesReference),
/// [Parameter](parameter::Parameter), [Priority](event::Priority), [RateRule](rule::RateRule),
/// [Reaction](reaction::Reaction), [Species](species::Species), [SpeciesReference](reaction::SpeciesReference),
/// [Trigger](event::Trigger), and [Unit](unit::Unit), plus the *id* attribute values of any SBML Level 3 package
/// element defined to be in the *SId* namespace of the [Model](model::Model).
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
/// **extentUnits** on [Model](model::Model)) must always conform to the syntax of the SBML
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
            let message = format!("XML namespace not declared for '{0}'.", element.full_name());
            issues.push(SbmlIssue::new_error(
                "10401",
                element.xml_element(),
                message,
            ))
        }
    }
}
