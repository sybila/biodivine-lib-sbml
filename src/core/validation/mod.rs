use crate::constants::element::{ALLOWED_ATTRIBUTES, ALLOWED_CHILDREN};
use crate::xml::{XmlElement, XmlWrapper};
use crate::{Sbml, SbmlIssue, SbmlIssueSeverity};
use std::collections::HashMap;
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

impl Sbml {
    /// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
    /// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
    /// elements or attributes placed in an SBML namespace do not conform to the SBML
    /// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
    pub(crate) fn apply_rule_10102(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.xml.read().unwrap();
        let rule_number = "10102";

        if doc.container().child_elements(doc.deref()).len() != 1 {
            issues.push(SbmlIssue {
                element: doc.container(),
                severity: SbmlIssueSeverity::Error,
                rule: rule_number.to_string(),
                message: "The document contains multiple root nodes.".to_string(),
            })
        }

        if let Some(root_element) = doc.root_element() {
            if root_element.name(doc.deref()) == "sbml" {
                validate_allowed_attributes(
                    rule_number,
                    root_element,
                    root_element.name(doc.deref()),
                    root_element.attributes(doc.deref()),
                    issues,
                );

                validate_allowed_children(
                    rule_number,
                    root_element,
                    root_element.name(doc.deref()),
                    root_element
                        .children(doc.deref())
                        .iter()
                        .map(|node| node.as_element().unwrap().full_name(doc.deref()))
                        .collect(),
                    issues,
                );
            } else {
                issues.push(SbmlIssue {
                    element: root_element,
                    severity: SbmlIssueSeverity::Error,
                    rule: rule_number.to_string(),
                    message: format!("Unknown root element <{}>", root_element.name(doc.deref())),
                })
            }
        }
    }
}

pub fn validate_allowed_attributes(
    rule: &str,
    element: Element,
    element_name: &str,
    attrs: &HashMap<String, String>,
    issues: &mut Vec<SbmlIssue>,
) {
    let allowed_attributes = ALLOWED_ATTRIBUTES.get(element_name).unwrap();

    for full_name in attrs.keys() {
        let (_prefix, attr_name) = Element::separate_prefix_name(full_name);
        if !allowed_attributes.contains(&attr_name) {
            issues.push(SbmlIssue {
                element,
                severity: SbmlIssueSeverity::Error,
                rule: rule.to_string(),
                message: format!(
                    "Unknown attribute [{}] at element <{}>",
                    attr_name, element_name
                ),
            })
        }
    }
}

pub fn validate_allowed_children(
    rule: &str,
    element: Element,
    element_name: &str,
    children_names: Vec<&str>,
    issues: &mut Vec<SbmlIssue>,
) {
    let allowed_children = ALLOWED_CHILDREN.get(element_name).unwrap();

    for child_full_name in children_names {
        let (_prefix, child_name) = Element::separate_prefix_name(child_full_name);
        if !allowed_children.contains(&child_name) {
            issues.push(SbmlIssue {
                element,
                severity: SbmlIssueSeverity::Error,
                rule: rule.to_string(),
                message: format!(
                    "Unknown child <{}> of element <{}>",
                    child_name, element_name
                ),
            })
        }
    }
}

pub fn apply_rule_10102(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let rule_number = "10102";
    let doc = xml_element.read_doc();
    let element = xml_element.raw_element();
    let attributes = element.attributes(doc.deref());
    let children_names = element
        .children(doc.deref())
        .iter()
        .map(|node| node.as_element().unwrap().full_name(doc.deref()))
        .collect();

    validate_allowed_attributes(
        rule_number,
        element,
        xml_element.tag_name().as_str(),
        attributes,
        issues,
    );
    validate_allowed_children(
        rule_number,
        element,
        xml_element.tag_name().as_str(),
        children_names,
        issues,
    );
}

pub(crate) fn get_allowed_children(xml_element: &XmlElement) -> &'static [&'static str] {
    let Some(allowed) = ALLOWED_CHILDREN.get(xml_element.tag_name().as_str()) else {
        return &[];
    };
    allowed
}
