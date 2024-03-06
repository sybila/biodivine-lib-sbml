use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use const_format::formatcp;
use regex::Regex;
use xml_doc::Element;

use crate::constants::element::{
    ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES, MATHML_ALLOWED_CHILDREN,
    REQUIRED_ATTRIBUTES, UNIQUE_CHILDREN,
};
use crate::constants::namespaces::{URL_MATHML, URL_SBML_CORE};
use crate::core::{BaseUnit, Model, SBase};
use crate::xml::{
    DynamicProperty, OptionalXmlProperty, XmlElement, XmlList, XmlProperty, XmlPropertyType,
    XmlWrapper,
};
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
mod unit;
mod unit_definition;
mod xml;

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

/// Performs very basic and the most critical sanity checks. more precisely:
/// - no invalid child elements are present in the model.
/// - the document contains all required children and attributes.
/// - each attribute value has correct type.
/// Any failing check is logged in *issues*.
pub(crate) fn sanity_check(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let attributes = xml_element.attributes();
    let element_name = xml_element.tag_name();

    apply_rule_10102_and_derivatives(xml_element, issues);

    if let Some(required) = REQUIRED_ATTRIBUTES.get(element_name.as_str()) {
        for req_attr in required.iter() {
            if !attributes.contains_key(&req_attr.to_string()) {
                let message = format!(
                    "Sanity check failed: missing required attribute [{req_attr}] on <{element_name}>."
                );
                let rule_id = tag_to_attribute_rule_id(element_name.as_str(), req_attr)
                    .unwrap_or("SANITY_CHECK");
                issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
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

/// Resolve tag name to attribute consistency rule. These are used when testing for missing
/// required or undeclared optional attributes.
fn tag_to_attribute_rule_id(tag_name: &str, attr_name: &str) -> Option<&'static str> {
    match tag_name {
        "sbml" => match attr_name {
            "level" => Some("20102"),
            "version" => Some("20103"),
            _ => Some("20108"),
        },
        "model" => Some("20222"),
        "listOfFunctionDefinitions" => Some("20223"),
        "listOfUnitDefinitions" => Some("20224"),
        "listOfCompartments" => Some("20225"),
        "listOfSpecies" => Some("20226"),
        "listOfParameters" => Some("20227"),
        "listOfInitialAssignments" => Some("20228"),
        "listOfRules" => Some("20229"),
        "listOfConstraints" => Some("20230"),
        "listOfReactions" => Some("20231"),
        "listOfEvents" => Some("20232"),
        "functionDefinition" => Some("20307"),
        "unitDefinition" => Some("20419"),
        "listOfUnits" => Some("20420"),
        "unit" => Some("20421"),
        "compartment" => Some("20517"),
        "species" => match attr_name {
            "compartment" => Some("20614"),
            _ => Some("20623"),
        },
        "parameter" => Some("20706"),
        "initialAssignment" => Some("20805"),
        "assignmentRule" => Some("20908"),
        "rateRule" => Some("20909"),
        "algebraicRule" => Some("20910"),
        "constraint" => Some("21009"),
        "reaction" => Some("21110"),
        "speciesReference" => Some("21116"),
        "modifierSpeciesReference" => Some("21117"),
        "listOfLocalParameters" => Some("21129"),
        "kineticLaw" => Some("21132"),
        "listOfReactants" | "listOfProducts" => Some("21150"),
        "listOfModifiers" => Some("21151"),
        "localParameter" => Some("21172"),
        "eventAssignment" => Some("21214"),
        "listOfEventAssignments" => Some("21224"),
        "event" => Some("21225"),
        "trigger" => Some("21226"),
        "delay" => Some("21227"),
        "priority" => Some("21232"),
        _ => None,
    }
}

/// Similar to [tag_to_attribute_rule_id], resolves a tag name into a rule ID which specifies
/// what child elements are allowed for that particular element.
fn tag_to_allowed_child_rule_id(tag_name: &str) -> Option<&'static str> {
    match tag_name {
        "listOfFunctionDefinitions" => Some("20206"),
        "listOfUnitDefinitions" => Some("20207"),
        "listOfCompartments" => Some("20208"),
        "listOfSpecies" => Some("20209"),
        "listOfParameters" => Some("20210"),
        "listOfInitialAssignments" => Some("20211"),
        "listOfRules" => Some("20212"),
        "listOfConstraints" => Some("20213"),
        "listOfReactions" => Some("20214"),
        "listOfEvents" => Some("20215"),
        "listOfUnits" => Some("20415"),
        "listOfReactants" | "listOfProducts" => Some("21104"),
        "listOfModifiers" => Some("21105"),
        "listOfEventAssignments" => Some("21223"),
        "listOfLocalParameters" => Some("21128"),
        _ => None,
    }
}

fn tag_to_unique_child_rule_id(tag_name: &str, child_name: &str) -> Option<&'static str> {
    // First, we catch the SBase-child issues. For the remaining elements, it depends.
    // In some cases, there is one general rule to catch all issues (like for `model`).
    // In other cases, each child has a separate rule defined in the specification
    // (such as for `event`).

    // Note that IDEA tends to get confused here and claims that some of these patterns are
    // unreachable, but that seems to be a bug in their static analysis and the actual tests
    // are passing without issues.
    match (tag_name, child_name) {
        (_, "annotation") => Some("10404"),
        (_, "notes") => Some("10805"),
        ("sbml", "model") => Some("20201"),
        // This is technically too broad, but the only other options seems to be to write down
        // all the list* objects, which is rather tedeous.
        ("model", _) => Some("20205"),
        ("functionDefinition", "math") => Some("20306"),
        ("unitDefinition", "listOfUnits") => Some("20414"),
        ("initialAssignment", "math") => Some("20804"),
        ("assignmentRule", "math") | ("rateRule", "math") | ("algebraicRule", "math") => {
            Some("20907")
        }
        ("constraint", "math") => Some("21007"),
        ("constraint", "message") => Some("21008"),
        // Same as `model` above, there is a lot of unique children to enumerate...
        ("reaction", _) => Some("21106"),
        ("kineticLaw", "listOfLocalParameters") => Some("21127"),
        ("kineticLaw", "math") => Some("21130"),
        ("event", "trigger") => Some("21201"),
        ("trigger", "math") => Some("21209"),
        ("delay", "math") => Some("21210"),
        ("event", "delay") => Some("21221"),
        ("event", "listOfEventAssignments") => Some("21222"),
        ("event", "priority") => Some("21230"),
        ("priority", "math") => Some("21231"),
        ("eventAssignment", "math") => Some("21213"),
        _ => None,
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
    let element = xml_list.xml_element();
    let name = element.tag_name();
    sanity_check(element, issues);

    if let Some(allowed) = ALLOWED_CHILDREN.get(name.as_str()) {
        // TODO:
        //      This is a minor hack which ensures we don't run sanity check on invalid child
        //      elements, as this is what SBML test suite seems to be doing. But it is not an
        //      explicit part of the specification as far as I can tell. Nevertheless, we are
        //      doing a similar thing in the `validate_list_of` method. In the future, it would
        //      be good if this part can be handled by the XmlList directly: i.e. we could get
        //      an iterator which goes through all *allowed* elements only.

        // Only sanity-check the allowed children. The rest is going to be reported as an
        // error and is probably malformed anyway.
        for object in xml_list.iter() {
            if !allowed.contains(&object.tag_name().as_str()) {
                // This is a linear-time check, which is not great, but the list of allowed
                // children is typically very short anyway.
                continue;
            }
            object.sanity_check(issues);
        }
    } else {
        // Sanity-check everyone.
        for object in xml_list.iter() {
            object.sanity_check(issues);
        }
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
        let (prefix, attr_name) = Element::separate_prefix_name(full_name);

        if !prefix.is_empty() {
            // According to the specification, the SBML core attributes should be placed in the
            // default empty namespace, with any additional attributes (e.g. added by packages)
            // placed in their respective namespaces. Hence, we can skip validating anything with
            // a non-empty prefix, as this is likely a non-core attribute. however...
            // TODO:
            //      If we find an attribute that has the core namespace but uses a prefix,
            //      we should report this as an error too, because it is technically out of spec.
            continue;
        }

        if !allowed_attributes.contains(&attr_name) {
            let message = format!(
                "An unknown attribute [{}] of the element <{}> found.",
                attr_name, element_name
            );
            // This is not written in the specification, but it seems that for validation of
            // attributes, the official implementation gives precedence to the more
            // element-specific rules, as opposed to using the generic "10102" rule ID.
            // We thus override the default ID if an element with a more specific rule
            // ID is detected.
            let rule_id =
                tag_to_attribute_rule_id(element_name.as_str(), attr_name).unwrap_or("10102");
            issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
        }
    }
}

/// Validates for a given element that its children (tag names) are only from predefined set of
/// children (tag names). If not, an error is logged in the vector of issues.
pub(crate) fn validate_allowed_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    let allowed_children = ALLOWED_CHILDREN.get(element_name.as_str()).unwrap();

    for child in xml_element.child_elements() {
        let child_name = child.tag_name();
        let child_namespace = child.namespace_url();
        if child_name == "math"
            && allowed_children.contains(&"math")
            && child_namespace != URL_MATHML
        {
            // A special case to handle rule 10201, in which a `math` element is found without
            // the proper MathML namespace. This only works if we are actually expecting a `math`
            // element at this position.
            let message = "Found a <math> element without the proper MathML namespace.".to_string();
            issues.push(SbmlIssue::new_error("10201", xml_element, message));
        } else if child_namespace == URL_SBML_CORE {
            // All other children are expected to be in the SBML Core namespace.
            if !allowed_children.contains(&child_name.as_str()) {
                let message = format!(
                    "An unknown child <{}> of the element <{}> found.",
                    child_name, element_name
                );
                let rule_id =
                    tag_to_allowed_child_rule_id(element_name.as_str()).unwrap_or("10102");
                issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
            }
        }
    }
}

/// Validates for a given element that its children that are required to appear at most once
/// indeed do. Logs error if this is violated.
pub(crate) fn validate_unique_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    let unique_children = UNIQUE_CHILDREN.get(element_name.as_str()).unwrap();

    let mut counts = HashMap::new();
    for child in xml_element.child_elements() {
        let child_name = child.tag_name();
        let child_namespace = child.namespace_url();
        if child_namespace == URL_SBML_CORE || child_namespace == URL_MATHML {
            let entry = counts.entry(child_name);
            let count = entry.or_insert(0usize);
            *count += 1;
        }
    }

    for name in *unique_children {
        if let Some(count) = counts.get(&name.to_string()) {
            if *count > 1 {
                let message = format!(
                    "Multiple instances of child <{}> found in element <{}>.",
                    name, element_name
                );
                let rule_id = tag_to_unique_child_rule_id(element_name.as_str(), name)
                    .unwrap_or("SANITY_CHECK");
                issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
            }
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
        xml::build_letter_group(),
        xml::build_digit_group(),
        xml::build_combining_char_group(),
        xml::build_extender_group(),
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

/// ### Rule 10102
/// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
/// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
/// elements or attributes placed in an SBML namespace do not conform to the SBML
/// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
fn apply_rule_10102_and_derivatives(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let doc = xml_element.read_doc();
    let element = xml_element.raw_element();
    let attributes = element
        .attributes(doc.deref())
        .keys()
        .map(|key| key.as_str())
        .collect::<Vec<&str>>();

    validate_allowed_attributes(xml_element, &attributes, issues);
    validate_allowed_children(xml_element, issues);
    validate_unique_children(xml_element, issues);
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

// TODO: might be placed inside SBASE validation method
/// ### Rule 10404
/// A given SBML element may contain at most *one* **Annotation** subobject.
pub(crate) fn apply_rule_10404(element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let annotation_elements = element.child_elements_filtered(|el| el.tag_name() == "annotation");

    if annotation_elements.len() > 1 {
        let message = format!(
            "Multiple annotation elements found in <{0}>.",
            element.tag_name()
        );
        issues.push(SbmlIssue::new_error("10404", element, message));
    }
}
