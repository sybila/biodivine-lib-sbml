use std::ops::Deref;

use xml_doc::Element;

use crate::{SbmlIssue, SbmlIssueSeverity};
use crate::constants::element::{MATHML_ALLOWED_CHILDREN_BY_ATTR, MATHML_ALLOWED_DEFINITION_URLS};
use crate::constants::namespaces::URL_MATHML;
use crate::core::Math;
use crate::core::validation::get_allowed_children;
use crate::xml::XmlWrapper;

impl Math {
    /// Applies rules:
    ///  - **[10201](self.apply_rule_10201)** - MathML content is permitted only within [Math] element.
    ///  - **[10202](self.apply_rule_10202)** - Validates list of permitted elements within [Math] element.
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        self.apply_rule_10201(issues);
        self.apply_rule_10202(issues);
        self.apply_rule_10203(issues);
        self.apply_rule_10204(issues);
        self.apply_rule_10205(issues);
    }

    /// ### Rule 10201
    /// is *partially* satisfied by the implementation of the rule
    /// [10102](crate::core::validation::apply_rule_10102) as we check each
    /// element present for its allowed children (except [Math] element that is
    /// the subject of this validation procedure) and thus **MathML** content
    /// can be present only within a [Math] element. However, additional check for
    /// explicit or implicit valid namespace of a [Math] element must be performed.
    fn apply_rule_10201(&self, issues: &mut Vec<SbmlIssue>) {
        if self.namespace_url() != URL_MATHML {
            issues.push(SbmlIssue {
                element: self.raw_element(),
                message: format!(
                    "Wrong namespace usage in a math element. Found {0}, but {1} should be used.",
                    self.namespace_url(),
                    URL_MATHML
                ),
                rule: "10201".to_string(),
                severity: SbmlIssueSeverity::Error,
            });
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10202
    /// Validates that only allowed subset of **MathML** child elements are present
    /// within [Math] element. An SBML package may allow new MathML elements to be
    /// added to this list, and if so, the package must define required="true" on
    /// the SBML container element <sbml>.
    fn apply_rule_10202(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children = self.raw_element().children_recursive(doc.deref());
        let allowed_children = get_allowed_children(self.xml_element());

        for child in children {
            if let Some(child_element) = child.as_element() {
                let child_tag_name = child_element.name(doc.deref());

                if !allowed_children.contains(&child_tag_name) {
                    issues.push(SbmlIssue {
                        element: child_element,
                        message: format!(
                            "Unknown child <{0}> of element <{1}>.",
                            child_tag_name, "math"
                        ),
                        rule: "10202".to_string(),
                        severity: SbmlIssueSeverity::Error,
                    });
                }
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10203
    /// In the SBML subset of MathML 2.0, the MathML attribute encoding is only permitted on
    /// **csymbol**, **annotation** and **annotation-xml**. No other MathML elements may have
    /// an encoding attribute. An SBML package may allow the encoding attribute on other
    /// elements, and if so, the package must define required=“true” on the SBML container element <sbml>.
    fn apply_rule_10203(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["encoding"];
        let children: Vec<Element> = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "encoding").is_some())
            .copied()
            .collect();

        for child in children {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [encoding] found on element <{0}>, which is forbidden. \
                        Attribute [encoding] is only permitted on <csymbol>, <annotation> and <annotation-xml>.",
                        name
                    ),
                    rule: "10203".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10204
    /// In the SBML subset of MathML 2.0, the MathML attribute definitionURL is only permitted on
    /// **ci**, **csymbol** and **semantics**. No other MathML elements may have a definitionURL attribute. An
    /// SBML package may allow the definitionURL attribute on other elements, and if so, the package
    /// must define required=“true” on the SBML container element <sbml>.
    fn apply_rule_10204(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["definitionURL"];
        let children: Vec<Element> = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "definitionURL").is_some())
            .copied()
            .collect();

        for child in children {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [definitionURL] found on element <{0}>, which is forbidden. \
                        Attribute [definitionURL] is only permitted on <ci>, <csymbol> and <semantics>.",
                        name
                    ),
                    rule: "10204".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10205
    /// In SBML Level 3, the only values permitted for the attribute definitionURL on a csymbol are
    /// “http://www.sbml.org/sbml/symbols/time”, “http://www.sbml.org/sbml/symbols/delay”,
    /// “http://www.sbml.org/sbml/symbols/avogadro”, and “http://www.sbml.org/sbml/symbols/rateOf”.
    /// An SBML package may allow new values for the definitionURL attribute of a csymbol, and if so,
    /// the package must define required=“true” on the SBML container element <sbml>.
    fn apply_rule_10205(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children: Vec<Element> = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| {
                child.attribute(doc.deref(), "definitionURL").is_some()
                    && child.name(doc.deref()) == "csymbol"
            })
            .copied()
            .collect();

        for child in children {
            let value = child.attribute(doc.deref(), "definitionURL").unwrap();
            if !MATHML_ALLOWED_DEFINITION_URLS.contains(&value) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!("Invalid definitionURL value found '{}'.", value),
                    rule: "10205".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }
}
