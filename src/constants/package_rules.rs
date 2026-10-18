//! The package rules that constrain the *attributes* of an element.
//!
//! An SBML package adds attributes to elements of the core (an `fbc:strict` on `<model>`) and
//! defines elements of its own with attributes in its own namespace (a `layout:compartment` on a
//! `<layout:compartmentGlyph>`). The specification states the permitted set per element, and the
//! crate's element tables express most of it; what they cannot express is the *absence* of an
//! attribute a package version defines but the element does not permit, because the tables are a
//! whitelist consulted by name and the type check does not look at a package attribute that no
//! accessor reads.
//!
//! This module is that missing half, as data: one row per rule, naming the element, the package
//! namespace, the attributes the element permits *in that namespace*, and the sentence of the
//! specification the report quotes. A row is the whole declaration of its rule:
//!
//!  * it is reported under its own rule id, and the id must be registered in the crate's rule table
//!    ([`RuleSet::CORE`](crate::RuleSet)), which is what keeps a row from inventing a rule;
//!  * its message is built from the specification sentence it carries, so the report reads as the
//!    rule does rather than as a generic "unknown attribute";
//!  * it is checked only for the elements it names, so a row cannot change the report of a document
//!    that carries no such element - which is what makes a row's blast radius reviewable (the
//!    corpus walk of the commit that adds it sees exactly that rule's documents).
//!
//! ## Why the constraint is a *whitelist per element* rather than a check on every package attribute
//!
//! The obvious implementation - report every attribute in a package namespace that the element's row
//! of the attribute tables does not list - was built, measured and removed: the element tables are
//! the *crate's* summary of several specifications, and their holes are not defects of the documents.
//! The measurement (a corpus walk, `notes/g42-report.md` section 5) found legitimate attributes that
//! no row names (`layout:originOfText` on a `<textGlyph>`, whose row spells it `layout:originText`;
//! `layout:metaIdRef` on a `<curveSegment>`, which has no row entry at all), so the wholesale form
//! reported valid documents. A per-rule row cannot do that: it names the element and the attributes
//! it permits, and it is added only where the specification states the constraint and the corpus can
//! check it.
use crate::constants::namespaces::Namespace;
use crate::xml::XmlElement;
use crate::SbmlIssue;

/// The attributes one element of a package permits, and the rule that says so.
///
/// The `permitted` list holds *local* names in the namespace of the element's package (so a row for
/// a Layout element lists `id`, not `layout:id`): the namespace is the row's `package` field, and
/// keeping the local name alone means the row does not repeat the package in every entry.
pub(crate) struct PackageAttributeRule {
    /// The rule id, which has to be a descriptor of the crate's rule table.
    pub(crate) id: &'static str,
    /// The tag name of the element the rule is about.
    pub(crate) element: &'static str,
    /// The namespace of the package that owns the element.
    pub(crate) package: Namespace,
    /// The attributes the element permits in the package's namespace, by local name.
    pub(crate) permitted: &'static [&'static str],
    /// The requirement, adapted from the specification's own sentence.
    pub(crate) message: &'static str,
}

/// Every package-attribute rule of this crate, one row per rule.
///
/// The list is ordered by rule id, and each row's documentation names the specification section the
/// permitted set comes from.
pub(crate) const PACKAGE_ATTRIBUTE_RULES: &[PackageAttributeRule] = &[
    // Layout, section 3.11: "A `<listOfSubGlyphs>` object may have the optional attributes `metaid`
    // and `sboTerm` defined by SBML Level 3 Core. No other attributes from the SBML Level 3 Core
    // namespace or the Layout namespace are permitted" - so the Layout namespace permits *no*
    // attribute here, and the two core ones are checked by the core rule for the element.
    PackageAttributeRule {
        id: "layout-20813",
        element: "listOfSubGlyphs",
        package: crate::constants::namespaces::NS_LAYOUT,
        permitted: &[],
        message: "A <listOfSubGlyphs> object may have the optional attributes 'metaid' and \
                  'sboTerm' defined by SBML Level 3 Core: no attribute of the Layout namespace is \
                  permitted on it.",
    },
    // FBC, section 3.8: "A `<listOfFluxObjectives>` object may have the optional 'metaid' and
    // 'sboTerm' defined by SBML Level 3 Core. No other attributes from the SBML Level 3 Core
    // namespace or the Flux Balance Constraints namespace are permitted on a
    // `<listOfFluxObjectives>` object."
    PackageAttributeRule {
        id: "fbc-20509",
        element: "listOfFluxObjectives",
        package: crate::constants::namespaces::NS_FBC,
        permitted: &[],
        message: "A <listOfFluxObjectives> object may have the optional 'metaid' and 'sboTerm' \
                  defined by SBML Level 3 Core; no attribute of the Flux Balance Constraints \
                  namespace is permitted on it.",
    },
    // FBC, section 3.4: "A SBML `<species>` object may have the optional attributes 'fbc:charge'
    // and 'fbc:chemicalFormula'. No other attributes from the Flux Balance Constraints namespaces
    // are permitted on a `<species>`."
    PackageAttributeRule {
        id: "fbc-20301",
        element: "species",
        package: crate::constants::namespaces::NS_FBC,
        permitted: &["charge", "chemicalFormula"],
        message: "A <species> object may carry the optional attributes 'fbc:charge' and \
                  'fbc:chemicalFormula'; no other attribute of the Flux Balance Constraints \
                  namespace is permitted on it.",
    },
    // FBC, section 3.11: "An SBML `<reaction>` object may have the optional attributes
    // 'fbc:lowerFluxBound' and 'fbc:upperFluxBound'. No other attributes from the Flux Balance
    // Constraints namespaces are permitted on a `<reaction>`."
    PackageAttributeRule {
        id: "fbc-20702",
        element: "reaction",
        package: crate::constants::namespaces::NS_FBC,
        permitted: &["lowerFluxBound", "upperFluxBound"],
        message: "A <reaction> object may carry the optional attributes 'fbc:lowerFluxBound' and \
                  'fbc:upperFluxBound'; no other attribute of the Flux Balance Constraints \
                  namespace is permitted on it.",
    },
    // FBC, section 3.6: "A `<geneProductAssociation>` object may have the optional attributes
    // 'fbc:id' and 'fbc:name'. No other attributes from the SBML Level 3 Flux Balance Constraints
    // namespace are permitted on a `<geneProductAssociation>` object."
    PackageAttributeRule {
        id: "fbc-20803",
        element: "geneProductAssociation",
        package: crate::constants::namespaces::NS_FBC,
        permitted: &["id", "name"],
        message: "A <geneProductAssociation> object may carry the optional attributes 'fbc:id' \
                  and 'fbc:name'; no other attribute of the Flux Balance Constraints namespace is \
                  permitted on it.",
    },
];

/// Reports the package attributes of `element` that its rules do not permit.
///
/// The check is one pass over the element's own attributes with no recursion, so it does not need a
/// cancellation check of its own: the caller
/// ([`internal_type_check`](crate::core::validation::type_check::internal_type_check)) runs inside
/// the cancellable walk of [`Sbml::validate`](crate::Sbml::validate), which checks the trigger at
/// every object it visits.
pub(crate) fn validate_package_attributes(element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let tag_name = element.tag_name();
    let raw = element.element();
    for rule in PACKAGE_ATTRIBUTE_RULES {
        if rule.element != tag_name.as_str() {
            continue;
        }
        for (name, _value) in raw.attributes() {
            let Some(namespace) = name.namespace() else {
                continue;
            };
            // Any *published version* of the package the row names: a document that uses FBC
            // Version 1 keeps the same attribute in the Version 1 URL, and the row states the
            // attribute set of the element, not of one version of it.
            if !crate::constants::namespaces::is_package_namespace_of(
                namespace.uri(),
                rule.package.0,
            ) {
                continue;
            }
            let local = name.local_name_str();
            if rule.permitted.contains(&local) {
                continue;
            }
            let message = format!(
                "The attribute [{}:{}] of the element <{tag_name}> is not permitted here. {}",
                rule.package.0, local, rule.message
            );
            issues.push(SbmlIssue::new(rule.id, element, message));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::namespaces::URL_PACKAGE_LAYOUT;
    use crate::{RuleSet, Sbml};

    /// The rule ids of a document's issues, for the tests that ask about a rule.
    fn issues_of(document: &str) -> Vec<String> {
        Sbml::read_str(document)
            .expect("The test document is well formed.")
            .validate()
            .expect("The tests register no cancellation trigger.")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect()
    }

    /// A document with one `<layout:listOfSubGlyphs>` carrying `attributes`.
    fn document_with_sub_glyphs(attributes: &str) -> String {
        format!(
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version1/core"
                     xmlns:layout="{URL_PACKAGE_LAYOUT}" level="3" version="1"
                     layout:required="false">
                 <model id="m">
                   <layout:listOfLayouts>
                     <layout:layout layout:id="l">
                       <layout:dimensions layout:width="1" layout:height="1"/>
                       <layout:listOfAdditionalGraphicalObjects>
                         <layout:generalGlyph layout:id="g">
                           <layout:boundingBox>
                             <layout:position layout:x="0" layout:y="0"/>
                             <layout:dimensions layout:width="1" layout:height="1"/>
                           </layout:boundingBox>
                           <layout:listOfSubGlyphs {attributes}/>
                         </layout:generalGlyph>
                       </layout:listOfAdditionalGraphicalObjects>
                     </layout:layout>
                   </layout:listOfLayouts>
                 </model>
               </sbml>"#
        )
    }

    #[test]
    fn a_layout_attribute_the_element_does_not_permit_is_reported_under_its_rule() {
        let issues = issues_of(&document_with_sub_glyphs(r#"layout:compartment="c""#));

        assert!(
            issues.contains(&"layout-20813".to_string()),
            "The Layout attribute must be reported under its rule: {issues:?}"
        );
    }

    #[test]
    fn the_core_attributes_the_element_permits_are_not_reported() {
        let issues = issues_of(&document_with_sub_glyphs(
            r#"metaid="m1" sboTerm="SBO:0000001""#,
        ));

        assert!(
            !issues.contains(&"layout-20813".to_string()),
            "The row permits the two core attributes: {issues:?}"
        );
    }

    #[test]
    fn every_row_names_a_registered_rule_and_an_element_of_its_package() {
        // A row is data, so the two things it can get wrong are its id (an id the crate cannot
        // report would panic at the first document that reaches it) and its element (a row for an
        // element the package does not own would never fire).
        for rule in super::PACKAGE_ATTRIBUTE_RULES {
            assert!(
                RuleSet::CORE.get(rule.id).is_some(),
                "The row for {} is not in the rule table.",
                rule.id
            );
            assert!(
                rule.id.starts_with(rule.package.0),
                "The row for {} does not belong to the package {} it names.",
                rule.id,
                rule.package.0
            );
            assert!(
                !rule.message.is_empty(),
                "The row for {} carries no message.",
                rule.id
            );
        }
    }
}
