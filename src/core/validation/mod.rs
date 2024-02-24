use crate::constants::element::{
    ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES, MATHML_ALLOWED_CHILDREN,
    REQUIRED_ATTRIBUTES,
};
use crate::constants::namespaces::URL_SBML_CORE;
use crate::core::SBase;
use crate::xml::{
    DynamicProperty, OptionalXmlProperty, XmlElement, XmlList, XmlProperty, XmlPropertyType,
    XmlWrapper,
};
use crate::{Sbml, SbmlIssue, SbmlIssueSeverity};
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
mod unit;
mod unit_definition;

/// Denotes an element that can be (and should be) validated against the SBML
/// validation rules.
pub(crate) trait SbmlValidable: XmlWrapper {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>);
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
            issues.push(SbmlIssue {
                element: doc.container(),
                message: "The document contains multiple root nodes. \
                Only one root <sbml> object is allowed."
                    .to_string(),
                rule: "10102".to_string(),
                severity: SbmlIssueSeverity::Error,
            })
        }

        if let Some(root_element) = doc.root_element() {
            if root_element.name(doc.deref()) == "sbml" {
                validate_allowed_attributes(
                    root_element,
                    root_element.name(doc.deref()),
                    &root_element
                        .attributes(doc.deref())
                        .keys()
                        .map(|key| key.as_str())
                        .collect::<Vec<&str>>(),
                    issues,
                );

                validate_allowed_children(
                    root_element,
                    root_element.name(doc.deref()),
                    &root_element
                        .children(doc.deref())
                        .iter()
                        .filter_map(|node| node.as_element().map(|it| it.full_name(doc.deref())))
                        .collect(),
                    issues,
                );
            } else {
                issues.push(SbmlIssue {
                    element: root_element,
                    message: format!(
                        "Invalid root element <{}> found.",
                        root_element.name(doc.deref())
                    ),
                    rule: "10102".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
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

    for req_attr in REQUIRED_ATTRIBUTES[element_name.as_str()] {
        if !attributes.contains_key(&req_attr.to_string()) {
            issues.push(SbmlIssue {
                element: xml_element.raw_element(),
                message: format!(
                    "Sanity check failed: missing required attribute [{0}] on <{1}>.",
                    req_attr, element_name
                ),
                rule: "SANITY_CHECK".to_string(),
                severity: SbmlIssueSeverity::Error,
            });
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
                match t.1 {
                    &"positive_int" => sanity_type_check::<u32>(attr_name, &xml_element, issues),
                    &"int" => sanity_type_check::<i32>(attr_name, &xml_element, issues),
                    &"double" => sanity_type_check::<f64>(attr_name, &xml_element, issues),
                    &"boolean" => sanity_type_check::<bool>(attr_name, &xml_element, issues),
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
        issues.push(SbmlIssue {
            element: xml_element.raw_element(),
            message: format!(
                "Sanity check failed: {0} On the attribute [{1}].",
                property.err().unwrap(),
                attribute_name
            ),
            rule: "SANITY_CHECK".to_string(),
            severity: SbmlIssueSeverity::Error,
        })
    }
}

pub(crate) fn sanity_check_of_list<T: SanityCheckable>(
    xml_list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
) {
    sanity_check(xml_list.xml_element(), issues);

    for object in xml_list.as_vec() {
        object.sanity_check(issues);
    }
}

pub(crate) fn validate_allowed_attributes(
    element: Element,
    element_name: &str,
    attributes: &Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let allowed_attributes = ALLOWED_ATTRIBUTES.get(element_name).unwrap();

    for full_name in attributes {
        let (_prefix, attr_name) = Element::separate_prefix_name(full_name);
        if !allowed_attributes.contains(&attr_name) {
            issues.push(SbmlIssue {
                element,
                message: format!(
                    "An unknown attribute [{}] of the element <{}> found.",
                    attr_name, element_name
                ),
                rule: "10102".to_string(),
                severity: SbmlIssueSeverity::Error,
            })
        }
    }
}

pub(crate) fn validate_allowed_children(
    element: Element,
    element_name: &str,
    children_names: &Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let allowed_children = ALLOWED_CHILDREN.get(element_name).unwrap();

    for child_full_name in children_names {
        let (_prefix, child_name) = Element::separate_prefix_name(child_full_name);
        if !allowed_children.contains(&child_name) {
            issues.push(SbmlIssue {
                element,
                message: format!(
                    "An unknown child <{}> of the element <{}> found.",
                    child_name, element_name
                ),
                rule: "10102".to_string(),
                severity: SbmlIssueSeverity::Error,
            })
        }
    }
}

pub(crate) fn validate_list_of_objects<T: SbmlValidable>(
    list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<String>,
) {
    let allowed = get_allowed_children(list.xml_element());
    apply_rule_10102(list.xml_element(), issues);
    apply_rule_10301(list.id().get(), list.xml_element(), issues, identifiers);

    for object in list.as_vec() {
        if allowed.contains(&object.tag_name().as_str()) {
            object.validate(issues, identifiers);
        }
    }
}

pub(crate) fn get_allowed_children(xml_element: &XmlElement) -> &'static [&'static str] {
    let Some(allowed) = ALLOWED_CHILDREN.get(xml_element.tag_name().as_str()) else {
        let Some(allowed) = MATHML_ALLOWED_CHILDREN.get(xml_element.tag_name().as_str()) else {
            return &[];
        };
        return allowed;
    };
    allowed
}

/// ### Rule 10102
/// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
/// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
/// elements or attributes placed in an SBML namespace do not conform to the SBML
/// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
pub(crate) fn apply_rule_10102(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let doc = xml_element.read_doc();
    let element = xml_element.raw_element();
    let element_name = xml_element.tag_name();
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

    validate_allowed_attributes(element, element_name.as_str(), &attributes, issues);
    validate_allowed_children(element, element_name.as_str(), &children_names, issues);
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
    if let Some(id) = id {
        if identifiers.contains(&id) {
            issues.push(SbmlIssue {
                element: xml_element.raw_element(),
                message: format!(
                    "The identifier ('{0}') of <{1}> is already present in the <model>.",
                    id,
                    xml_element.tag_name()
                ),
                rule: "10301".to_string(),
                severity: SbmlIssueSeverity::Error,
            })
        } else {
            identifiers.insert(id);
        }
    }
}
