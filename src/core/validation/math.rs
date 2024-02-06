use crate::constants::namespaces::URL_MATHML;
use crate::core::validation::get_allowed_children;
use crate::core::Math;
use crate::xml::XmlWrapper;
use crate::{SbmlIssue, SbmlIssueSeverity};
use std::ops::Deref;

impl Math {
    /// ### Rule 10201
    /// is *partially* satisfied by the implementation of the rule
    /// [10102](crate::core::validation::apply_rule_10102) as we check each
    /// element present for its allowed children (except [Math] element that is
    /// the subject of this validation procedure) and thus **MathML** content
    /// can be present only within a [Math] element. However, additional check for
    /// explicit or implicit valid namespace of a [Math] element must be performed.  
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        self.apply_rule_10201(issues);
        self.apply_rule_10202(issues);
    }

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
}
