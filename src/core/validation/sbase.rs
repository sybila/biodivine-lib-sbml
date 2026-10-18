use crate::core::validation::{
    check_identifier_uniqueness, is_level_3_version_1, matches_sboterm_pattern,
    matches_sid_pattern, matches_xml_id_pattern, matches_xml_string_pattern,
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
/// [Compartment](crate::core::Compartment), [Constraint](crate::core::Constraint), [Delay](crate::core::Delay),
/// [Event](crate::core::Event), [EventAssignment](crate::core::EventAssignment),
/// [FunctionDefinition](crate::core::FunctionDefinition),
/// [InitialAssignment](crate::core::InitialAssignment), [KineticLaw](crate::core::KineticLaw),
/// [ListOfCompartments](crate::core::Model::compartments), [ListOfConstraints](crate::core::Model::constraints),
/// [ListOfEventAssignments](crate::core::Event::event_assignments), [ListOfEvents](crate::core::Model::events),
/// [ListOfFunctionDefinitions](crate::core::Model::function_definitions),
/// [ListOfInitialAssignments](crate::core::Model::initial_assignments),
/// [ListOfLocalParameters](crate::core::KineticLaw::local_parameters),
/// [ListOfModifierSpeciesReferences](crate::core::Reaction::modifiers), [ListOfParameters](crate::core::Model::parameters),
/// [ListOfReactions](crate::core::Model::reactions), [ListOfRules](crate::core::Model::rules),
/// [ListOfSpecies](crate::core::Model::species), [ListOfSpeciesReferences](crate::core::Reaction::reactants),
/// [ListOfUnitDefinitions](crate::core::Model::unit_definitions), [ListOfUnits](crate::core::UnitDefinition::units),
/// [Model](crate::core::Model), [ModifierSpeciesReference](crate::core::ModifierSpeciesReference),
/// [Parameter](crate::core::Parameter), [Priority](crate::core::Priority), [RateRule](crate::core::RateRule),
/// [Reaction](crate::core::Reaction), [Species](crate::core::Species), [SpeciesReference](crate::core::SpeciesReference),
/// [Trigger](crate::core::Trigger), and [Unit](crate::core::Unit), plus the *id* attribute values of any SBML Level 3 package
/// element defined to be in the *SId* namespace of the [Model](crate::core::Model).
pub(crate) fn apply_rule_10301(
    id: Option<SId>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<SId>,
) {
    // The rule names its classes explicitly, and it is not a rule about "every object with an id":
    // a `UnitDefinition` has its own rule (10302), a `LocalParameter` has its own (10303), and an
    // object of a class the list does not name is not covered by this rule at all. Reporting an id
    // conflict on such an object is a false positive - the corpus has 30 of them, all of the shape
    // "two unit definitions or two local parameters share an id" - so the class is checked first.
    // The class list is a property of the document's version: the Level 3 Version 1 text names nine
    // classes and the Version 2 text names the longer list below, so a Version 1 document is
    // checked against its own list (a conflict on a class only Version 2 names is not a 10301
    // there).
    let classes = if is_level_3_version_1(xml_element) {
        RULES_10301_CLASSES_L3V1
    } else {
        RULES_10301_CLASSES
    };
    if !classes.contains(&xml_element.tag_name().as_str()) {
        return;
    }
    check_identifier_uniqueness("10301", "id", id, xml_element, issues, identifiers);
}

/// The classes whose `id` has to be unique across a model, as L3V2 rule 10301 names them.
///
/// The list is the rule text of the specification (the Level 3 Version 2 class list; the Level 3
/// Version 1 text names a shorter one, which is a version difference for the versioning work). It
/// contains the container classes as well as the object classes, because the rule names both, and
/// it deliberately does not contain `UnitDefinition`, `LocalParameter` or `LocalSpeciesReference`:
/// they have rules of their own (10302 and 10303) and are therefore not part of this one.
/// The classes of L3V1 rule 10301, which names nine of them: `Model`, `FunctionDefinition`,
/// `Compartment`, `Species`, `Reaction`, `SpeciesReference`, `ModifierSpeciesReference`, `Event` and
/// `Parameter`. A class the Version 2 list adds (a rule, a constraint, a delay, the container
/// classes, ...) is not part of the Version 1 rule, so a Version 1 document is not reported for a
/// conflict on one of them.
const RULES_10301_CLASSES_L3V1: &[&str] = &[
    "model",
    "functionDefinition",
    "compartment",
    "species",
    "reaction",
    "speciesReference",
    "modifierSpeciesReference",
    "event",
    "parameter",
];

const RULES_10301_CLASSES: &[&str] = &[
    "algebraicRule",
    "assignmentRule",
    "compartment",
    "constraint",
    "delay",
    "event",
    "eventAssignment",
    "functionDefinition",
    "initialAssignment",
    "kineticLaw",
    "listOfCompartments",
    "listOfConstraints",
    "listOfEventAssignments",
    "listOfEvents",
    "listOfFunctionDefinitions",
    "listOfInitialAssignments",
    "listOfLocalParameters",
    "listOfModifierSpeciesReferences",
    "listOfParameters",
    "listOfReactions",
    "listOfRules",
    "listOfSpecies",
    "listOfSpeciesReferences",
    "listOfUnitDefinitions",
    "listOfUnits",
    "model",
    "modifierSpeciesReference",
    "parameter",
    "priority",
    "rateRule",
    "reaction",
    "species",
    "speciesReference",
    "trigger",
    "unit",
];

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
        issues.push(SbmlIssue::new("10308", xml_element, message))
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
        issues.push(SbmlIssue::new("10309", xml_element, message))
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
        issues.push(SbmlIssue::new("10310", xml_element, message))
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
        issues.push(SbmlIssue::new("10312", xml_element, message))
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
            issues.push(SbmlIssue::new("10401", element.xml_element(), message))
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
            issues.push(SbmlIssue::new("10402", &element, message));
        } else {
            unique_namespaces.insert(namespace);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Sbml;
    use std::collections::BTreeSet;

    /// The rule ids of a report, so an assertion names the rule it is about.
    fn rules_of(document: &str) -> BTreeSet<String> {
        Sbml::read_str(document)
            .expect("the document is well-formed XML")
            .validate()
            .expect("validation cannot be cancelled")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect()
    }

    /// ### Rule 10301
    /// All the identifiers of the model must be unique across the model, for the object classes the
    /// rule names.
    ///
    /// ### Rule 10302
    /// The identifier of a `UnitDefinition` must be unique within the model, and a `UnitDefinition`
    /// is *not* in the class list of 10301 (it has its own rule), so two unit definitions that share
    /// an identifier are a 10302 and never a 10301.
    #[test]
    fn two_unit_definitions_with_one_identifier_are_10302_and_not_10301() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model>
    <listOfUnitDefinitions>
      <unitDefinition id="u"><listOfUnits><unit kind="second"/></listOfUnits></unitDefinition>
      <unitDefinition id="u"><listOfUnits><unit kind="second"/></listOfUnits></unitDefinition>
    </listOfUnitDefinitions>
  </model>
</sbml>"#;
        let rules = rules_of(document);
        assert!(rules.contains("10302"), "expected 10302 in {rules:?}");
        assert!(
            !rules.contains("10301"),
            "10301 must not cover a UnitDefinition (rule 10302 owns it): {rules:?}"
        );
    }

    /// ### Rule 10301, the version difference
    /// The Level 3 Version 1 text of the rule names nine classes and the Version 2 text a longer
    /// list, so a conflict on a class only Version 2 names (here an `AssignmentRule`) is a 10301 in
    /// a Version 2 document and *not* a 10301 in a Version 1 one. Both documents are otherwise
    /// identical, so the version is the only difference.
    #[test]
    fn the_class_list_of_10301_follows_the_document_version() {
        let rules_of_version = |version: &str| {
            let document = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version{version}/core" level="3" version="{version}">
  <model>
    <listOfParameters><parameter id="p" constant="true"/></listOfParameters>
    <listOfRules>
      <assignmentRule id="r" variable="p"><math xmlns="http://www.w3.org/1998/Math/MathML"><ci>p</ci></math></assignmentRule>
      <assignmentRule id="r" variable="p"><math xmlns="http://www.w3.org/1998/Math/MathML"><ci>p</ci></math></assignmentRule>
    </listOfRules>
  </model>
</sbml>"#
            );
            rules_of(&document)
        };
        assert!(
            rules_of_version("2").contains("10301"),
            "L3V2 10301 names AssignmentRule"
        );
        assert!(
            !rules_of_version("1").contains("10301"),
            "L3V1 10301 does not name AssignmentRule"
        );
    }

    /// Two compartments that share an identifier are a 10301: `Compartment` is one of the classes
    /// the rule names, so the report has to be there.
    #[test]
    fn two_compartments_with_one_identifier_are_10301() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model>
    <listOfCompartments>
      <compartment id="c" constant="true"/>
      <compartment id="c" constant="true"/>
    </listOfCompartments>
  </model>
</sbml>"#;
        let rules = rules_of(document);
        assert!(rules.contains("10301"), "expected 10301 in {rules:?}");
    }
}
