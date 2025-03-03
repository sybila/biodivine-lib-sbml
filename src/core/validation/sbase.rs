use crate::core::validation::{
    check_identifier_uniqueness, matches_sboterm_pattern, matches_sid_pattern,
    matches_xml_id_pattern, matches_xml_string_pattern,
};
use crate::core::{MetaId, SBase, SId};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlElement, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

/// Validation of rules that are relevant to *all* objects that implement [SBase].
pub(crate) fn validate_sbase<T: SBase>(
    object: &T,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<SId>,
    meta_ids: &mut HashSet<MetaId>,
) {
    let xml_element = object.xml_element();
    let id = object.id();
    let meta_id = object.meta_id();
    let sbo_term = object.sbo_term();
    let name = object.name();

    // These rules have to be checked first, because they validate the format of each attribute.
    apply_rule_10310(id.get_raw(), xml_element, issues);
    apply_rule_10309(meta_id.get_raw(), xml_element, issues);
    apply_rule_10308(sbo_term.get_raw(), xml_element, issues);
    apply_rule_10312(name.get(), xml_element, issues);

    apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
    if let Ok(id) = id.get_checked() {
        apply_rule_10301(id, xml_element, issues, identifiers);
    }

    if let Some(annotation) = object.annotation().get() {
        apply_rule_10401(&annotation, issues);
        apply_rule_10402(&annotation, issues);
    }
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
    id: Option<SId>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<SId>,
) {
    check_identifier_uniqueness("10301", "id", id, xml_element, issues, identifiers);
}

/// ### Rule 10307
/// Every *metaid* attribute value must be unique across the set of all *metaid* values in a model.
pub(crate) fn apply_rule_10307(
    meta_id: Option<MetaId>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    meta_ids: &mut HashSet<MetaId>,
) {
    check_identifier_uniqueness("10307", "meta_id", meta_id, xml_element, issues, meta_ids);
}

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
