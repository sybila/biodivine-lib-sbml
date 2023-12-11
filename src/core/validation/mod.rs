use crate::xml::OptionalXmlChild;
use crate::{Sbml, SbmlIssue, SbmlIssueSeverity};
use std::ops::Deref;

mod model;

impl Sbml {
    /// An SBML XML document must not contain undefined elements or attributes in the SBML Level 3
    /// Core namespace or in a SBML Level 3 package namespace. Documents containing unknown
    /// elements or attributes placed in an SBML namespace do not conform to the SBML
    /// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
    pub(crate) fn apply_rule_10102(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.xml.read().unwrap();
        let rule_number = "10102".to_string();

        if doc.root_nodes().len() != 1 {
            issues.push(SbmlIssue {
                element: doc.container(),
                severity: SbmlIssueSeverity::Error,
                rule: rule_number.clone(),
                message: "The document contains multiple root nodes.".to_string(),
            })
        }

        if let Some(root_element) = doc.root_element() {
            if root_element.name(doc.deref()) == "sbml" {
                if let Some(model) = self.model().get() {
                    model.apply_rule_10102(issues);
                }
            } else {
                issues.push(SbmlIssue {
                    element: root_element,
                    severity: SbmlIssueSeverity::Error,
                    rule: rule_number,
                    message: "Root element is invalid".to_string(),
                })
            }
        }
    }
}
