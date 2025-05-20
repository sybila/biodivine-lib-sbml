use crate::constants::element::{
    namespace_for_prefix, ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN, ATTRIBUTE_TYPES,
    REQUIRED_ATTRIBUTES, REQUIRED_CHILDREN, UNIQUE_CHILDREN,
};
use crate::constants::namespaces::{URL_MATHML, URL_PACKAGE_FBC, URL_SBML_CORE};
use crate::constraint::FbcType;
use crate::core::SId;
use crate::qual::{Sign, TransitionInputEffect, TransitionOutputEffect};
use crate::xml::{
    OptionalSbmlProperty, SbmlProperty, XmlElement, XmlList, XmlProperty, XmlPropertyType,
    XmlWrapper,
};
use crate::SbmlIssue;
use biodivine_xml_doc::Element;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

/// Denotes an element that possess a way to self-test against the most critical checks. This
/// should be executed **before** the actual document validation. If the type check fails,
/// validation is likely to panic and crash the program.
///
/// Once a document is type checked, performing only safe operations on it should always keep
/// it in a type checked state.
pub(crate) trait CanTypeCheck: XmlWrapper {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
    }
}

/// Implements basic integrity checks that are necessary to enable full validation.
/// In general, these should ensure that our XML abstractions work as expected and the model
/// is safe to work with. In particular:
///  - Only allowed properties are present.
///  - All required properties are set.
///  - All properties have the correct type.
///  - Only allowed child elements are present (including in XML lists).
///  - All required child elements are present.
///  - All relevant child elements appear only once in their parents.
///  - Namespaces are correctly applied (in particular, rule 10201).
///
/// Most of these checks are technically covered by rule 10102, but for most SBML elements, a more
/// specific rule ID exists as well. In order to avoid implementing all these rules
/// independently, we perform a single recursive "type check" procedure that then maps
/// any discovered issues to the correct rule ID (and defaults to 10102 or other appropriate rule
/// when an element-specific rule does not exist).
///
/// Any failing check is logged into `issues`.
pub(crate) fn internal_type_check(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let attributes = xml_element.attributes();
    let element_name = xml_element.tag_name();

    // Checks that:
    //  - Only allowed attributes are present.
    //  - Only allowed children are present.
    //  - All required children are present.
    //  - Each allowed child is present at most once.
    apply_rule_10102_and_derivatives(xml_element, issues);

    // Check that all required attributes are present.
    if let Some(required) = REQUIRED_ATTRIBUTES.get(element_name.as_str()) {
        for req_attr in required.iter() {
            let (prefix, name) = Element::separate_prefix_name(req_attr);
            let namespace = namespace_for_prefix(prefix);
            let property = SbmlProperty::<String>::new(xml_element, name, namespace, namespace);
            if !property.is_set() {
                let message = format!(
                    "Sanity check failed: missing required attribute [{req_attr}] on <{element_name}>."
                );
                let rule_id = tag_to_attribute_rule_id(element_name.as_str(), req_attr)
                    .unwrap_or("SANITY_CHECK");
                issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
            }
        }
    }

    // Typecheck all relevant attributes.
    for attr in attributes {
        let attr_name = attr.0.as_str();
        let (_prefix, name) = Element::separate_prefix_name(attr_name);
        let Some(types) = ATTRIBUTE_TYPES.get(element_name.as_str()) else {
            break;
        };

        // t => (attribute name, attribute value)
        for (attr_id, attr_type) in types {
            let (_prefix, name2) = Element::separate_prefix_name(attr_name);
            // TODO:
            //  This ignores namespace prefixes as simply assumes that if we find the
            //  right name, it is the specified attribute.
            if name == name2 {
                match *attr_type {
                    "positive_int" => type_check_of_property::<u32>(attr_id, xml_element, issues),
                    "int" => type_check_of_property::<i32>(attr_id, xml_element, issues),
                    "double" => type_check_of_property::<f64>(attr_id, xml_element, issues),
                    "boolean" => type_check_of_property::<bool>(attr_id, xml_element, issues),
                    "sid" => type_check_of_property::<SId>(attr_id, xml_element, issues),
                    "fbc_type" => type_check_of_property::<FbcType>(attr_id, xml_element, issues),
                    "sign" => type_check_of_property::<Sign>(attr_id, xml_element, issues),
                    "input_effect" => type_check_of_property::<TransitionInputEffect>(
                        attr_id,
                        xml_element,
                        issues,
                    ),
                    "output_effect" => type_check_of_property::<TransitionOutputEffect>(
                        attr_id,
                        xml_element,
                        issues,
                    ),
                    _ => (),
                }
            };
        }
    }
}

/// A utility function to perform a type check on an [XmlList] instance and all its valid
/// child elements.
pub(crate) fn type_check_of_list<T: CanTypeCheck>(
    xml_list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
) {
    let element = xml_list.xml_element();
    let name = element.tag_name();
    internal_type_check(element, issues);

    if let Some(allowed) = ALLOWED_CHILDREN.get(name.as_str()) {
        // Only sanity-check the allowed children. The rest is going to be reported as an
        // error and is probably malformed anyway.
        for object in xml_list.iter() {
            if !allowed.contains(&object.tag_name().as_str()) {
                // This is a linear-time check, which is not great, but the list of allowed
                // children is typically very short anyway.
                continue;
            }
            object.type_check(issues);
        }
    } else {
        // Sanity-check everyone.
        for object in xml_list.iter() {
            object.type_check(issues);
        }
    }
}

/// Performs a type check of a value of a specific attribute.
/// If check fails, error is logged in *issues*.
fn type_check_of_property<T: XmlPropertyType>(
    attribute_name: &'static str,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    let (prefix, name) = Element::separate_prefix_name(attribute_name);
    let namespace = namespace_for_prefix(prefix);
    let property = OptionalSbmlProperty::<T>::new(xml_element, name, namespace, namespace);
    if let Some(err) = property.get_checked().err() {
        // TODO:
        //  This also maps to a lot of concrete rule IDs based on the tag/attribute and
        //  will need a separate method to resolve.
        let message = format!(
            "Sanity check failed: {0} On the attribute [{1}].",
            err, attribute_name
        );
        issues.push(SbmlIssue::new_error("SANITY_CHECK", xml_element, message));
    }
}

/// ### Rule 10102
/// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
/// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
/// elements or attributes placed in an SBML namespace do not conform to the SBML
/// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
///
/// Internally, this can also resolve to a more specific rule ID if such rule exists.
/// See [tag_to_allowed_child_rule_id], [tag_to_unique_child_rule_id], and
/// [tag_to_attribute_rule_id] for the exact mapping between rule IDs and problematic tags.
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
    validate_required_children(xml_element, issues);
    validate_unique_children(xml_element, issues);
}

/// Validates for a given element that its attributes (keys) are only from predefined set of
/// attributes (keys). If not, an error is logged in the vector of issues.
pub fn validate_allowed_attributes(
    xml_element: &XmlElement,
    attributes: &Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let element_name = xml_element.tag_name();
    let Some(allowed_attributes) = ALLOWED_ATTRIBUTES.get(element_name.as_str()) else {
        // This might happen if someone runs this method on a tag in the SBML namespace that
        // is not officially supported. In such a case, other functions should report that error,
        // but we just silently continue.
        return;
    };

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

            let rule_id =
                tag_to_attribute_rule_id(element_name.as_str(), attr_name).unwrap_or("10102");
            issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
        }
    }
}

/// Validates for a given element that its children (tag names) are only from the predefined set
/// of children (tag names). If not, an error is logged in the vector of issues.
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
        } else if child_namespace == URL_SBML_CORE || child_namespace == URL_PACKAGE_FBC {
            // All other children are expected to be in the SBML Core namespace. Anything else
            // that is not in the core namespace is skipped.
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

pub(crate) fn validate_required_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    let Some(required_children) = REQUIRED_CHILDREN.get(element_name.as_str()) else {
        return;
    };

    let mut child_tag_names = HashSet::new();
    for child in xml_element.child_elements() {
        let child_name = child.tag_name();
        child_tag_names.insert(child_name);
    }

    for required in required_children.iter() {
        if !child_tag_names.contains(*required) {
            let message = format!(
                "Missing required child <{}> of the element <{}>.",
                required, element_name
            );
            // TODO: Check if these rule IDs are ok.
            let rule_id = tag_to_allowed_child_rule_id(element_name.as_str()).unwrap_or("10102");
            issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
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
            // Right now, we are only testing core and math elements.
            let entry = counts.entry(child_name);
            let count = entry.or_insert(0usize);
            *count += 1;
        }
    }

    for name in *unique_children {
        if let Some(count) = counts.get(*name) {
            if *count > 1 {
                let message = format!(
                    "Multiple instances of child <{}> found in element <{}>.",
                    name, element_name
                );
                let rule_id =
                    tag_to_unique_child_rule_id(element_name.as_str(), name).unwrap_or("10102");
                issues.push(SbmlIssue::new_error(rule_id, xml_element, message));
            }
        }
    }
}

/// Resolve tag name to attribute consistency rule. These are used when testing for missing,
/// required, or undeclared optional attributes.
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
            "chemicalFormula" | "charge" => Some("fbc-20301"),
            _ => Some("20623"),
        },
        "parameter" => Some("20706"),
        "initialAssignment" => Some("20805"),
        "assignmentRule" => Some("20908"),
        "rateRule" => Some("20909"),
        "algebraicRule" => Some("20910"),
        "constraint" => Some("21009"),
        "reaction" => match attr_name {
            "lowerFluxBound" | "upperFluxBound" => Some("fbc-20703"),
            _ => Some("21110"),
        },
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
        //fbc package
        "objective" => Some("fbc-20503"),
        "geneProductAssociation" => Some("fbc-2803"),
        "fluxObjective" => Some("fbc-20603"),
        "geneProductRef" => Some("fbc-20903"),
        "geneProduct" => Some("fbc-21203"),
        //qual package
        "qualitativeSpecies" => Some("qual-20303"),
        "transition" => Some("qual-20403"),
        "listOfInputs" => Some("qual-20410"),
        "listOfOutputs" => Some("qual-20411"),
        "listOfFunctionTerms" => Some("qual-20412"),
        "input" => Some("qual-20503"),
        "output" => Some("qual-20603"),
        "defaultTerm" => Some("qual-20703"),
        "functionTerm" => Some("qual-20803"),
        _ => None,
    }
}

/// Similar to [tag_to_attribute_rule_id], resolves a tag name into a rule ID which specifies
/// what child elements are allowed for that particular element.
/// TODO: Add elements from packages...
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
        "boundingBox" => Some("21303"),

        //fbc  package
        "listOfFluxObjectives" => Some("fbc-20508"),
        "geneProductAssociation" => Some("fbc-20805"),
        //qual package
        "transition" => Some("20406"),
        "listOfInputs" => Some("qual-20407"),
        "listOfOutputs" => Some("qual-20408"),
        "listOfFunctionTerms" => Some("qual-20409"),
        "functionTerm" => Some("qual-20804"),

        _ => None,
    }
}

/// Similar to [tag_to_attribute_rule_id], resolves a combination of parent and child tags
/// into a rule ID which specifies that the child must be unique in the parent.
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
        ("transition", "listOfFunctionTerms") | ("transition", "listOfInputs") | ("transition", "listOfOutputs") => Some("qual-20405"),
        _ => None,
    }
}
