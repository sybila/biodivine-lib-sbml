use const_format::formatcp;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use crate::cancellation::CancellationCheck;
use crate::constants::deltas;
use crate::constants::version::document_level_version_or_base;
pub(crate) use crate::constants::version::is_level_3_version_1;
use crate::core::sbase::SId;
use crate::core::validation::sbase::validate_sbase;
use crate::core::{BaseUnit, MetaId, Model};
use crate::xml::XmlElement;
use crate::xml::XmlList;
use crate::xml::{SbmlObject, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;

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
pub(crate) mod sbase;
mod species;
use crate::core::validation::type_check::type_check_problems;

/// This module implements basic integrity checks that are necessary to enable full validation.
/// In general, these should ensure that our XML abstractions work as expected and the model
/// is safe to work with. In particular:
///  - All required properties are set.
///  - All properties have the correct type.
///  - All required child elements are present.
///  - All child elements appear only once in their parents.
///  - All XML lists only contain children of the allowed type.
///  - Namespaces are correctly applied.
///
/// Most of these checks are technically covered by rule 10102, but for most SBML elements, a more
/// specific rule ID exists as well. In order to avoid implementing all these rules
/// independently, we perform a single recursive "type check" procedure that then maps
/// any discovered issues to the correct rule ID (and defaults to 10102 or other appropriate rule
/// when an element-specific rule does not exist).
pub(crate) mod type_check;
mod unit;
mod unit_definition;
mod xml_definitions;

/// Denotes an element that can be (and should be) validated against the SBML
/// validation rules.
pub(crate) trait SbmlValidable: XmlWrapper {
    /// Walk this object and its SBML children, unless its typed view is unusable.
    ///
    /// This method is the one entry point of the object walk, and it is a *guard*: it asks the
    /// element-local type check of [`type_check_problems`] whether the object's own element has a
    /// problem - the type check is what tells the walk which typed reads are safe - and, if it has,
    /// it reports one `not-validated` issue for the object and returns without descending. The
    /// check is not skipped for the whole document any more: the walk of a document reaches every
    /// object whose own element is type-consistent, and the objects it does not reach are named in
    /// the report.
    ///
    /// A `check` is handed down explicitly rather than read from a thread-local, so that the
    /// countdown of a document walk cannot be reset by a nested walk; the guard accounts for the
    /// object with [`CancellationCheck::tick`](crate::cancellation::CancellationCheck::tick) (and
    /// every implementation accounts for itself again, which is harmless).
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of
    /// the current thread has fired.
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        let problems = type_check_problems(self.xml_element());
        if !problems.is_empty() {
            let element = self.xml_element().tag_name();
            let count = problems.len();
            let first = problems
                .first()
                .map(|issue| issue.rule())
                .unwrap_or("type check");
            let message = format!(
                "The element <{element}> was not validated, together with the elements below it: \
                 its type check reported {count} problem(s), the first of them rule {first}. Fix \
                 the report of that rule to have this part of the document validated."
            );
            issues.push(SbmlIssue::new("not-validated", self.xml_element(), message));
            return Ok(());
        }
        self.validate_object(issues, identifiers, meta_ids, check)
    }

    /// Check this object and its SBML children against the rules of the specification.
    ///
    /// The implementations of this method are the walk itself; a caller outside the walk uses
    /// [`SbmlValidable::validate`], which is what guards them.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of
    /// the current thread has fired.
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()>;
}

/// Executes a validation of xml list object itself and all its children.
///
/// The list container is a visited object of the walk as well, so it accounts for itself here and
/// then leaves the accounting of every element to the leading tick of the element's own
/// [`SbmlValidable::validate`] implementation.
///
/// # Errors
///
/// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of the
/// current thread has fired.
pub(crate) fn validate_list_of_objects<T: SbmlValidable + SbmlObject>(
    list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<SId>,
    meta_ids: &mut HashSet<MetaId>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    check.tick()?;
    validate_sbase(list, issues, identifiers, meta_ids);

    let allowed = get_allowed_children(list.xml_element());
    for object in list.as_vec() {
        // Two filters, because either of them alone would hand an element to a wrapper that does
        // not describe it: the allowed-child table of the container (a child the container permits)
        // and the identity of the item type (the wrapper the container declares. The SBase payloads
        // `notes` and `annotation` pass the first test - every container permits them - but they are
        // not items, so validating them as one would read properties they cannot have.
        if allowed.contains(&object.xml_element().tag_name().as_str())
            && T::matches_element(object.xml_element())
        {
            object.validate(issues, identifiers, meta_ids, check)?;
        }
    }

    Ok(())
}

/// The children the given element permits, in the version of the document it belongs to.
///
/// The lookup is the core element table first and the MathML subset second, exactly as before; both
/// are now version-aware, so the Level 3 Version 1 subset of rule 10202 (the Version 2 one without
/// the five operators Version 2 added) comes out of the MathML delta rather than out of a
/// hand-maintained copy of the whole list.
pub(crate) fn get_allowed_children(xml_element: &XmlElement) -> Cow<'static, [&'static str]> {
    let tag_name = xml_element.tag_name();
    let version = document_level_version_or_base(xml_element);
    if let Some(allowed) = deltas::allowed_children(&tag_name, version) {
        allowed
    } else if let Some(allowed) = deltas::mathml_children(&tag_name, version) {
        allowed
    } else {
        Cow::Borrowed(&[])
    }
}

/// Checks that a given identifier is unique in the given set of identifiers. If the identifier
/// is unique, it is included in the given set of identifiers, otherwise error is logged in the
/// vector of issues.
fn check_identifier_uniqueness<ID: Eq + Hash + Display>(
    rule: &str,
    attr_name: &str,
    identifier: Option<ID>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
    identifiers: &mut HashSet<ID>,
) {
    if let Some(identifier) = identifier {
        if identifiers.contains(&identifier) {
            let tag_name = xml_element.tag_name();
            let message = format!(
                "The {attr_name} ('{identifier}') of <{tag_name}> is already present in the <model>."
            );
            issues.push(SbmlIssue::new(rule, xml_element, message));
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
pub(crate) fn matches_sid_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **SBOTerm** syntax.
pub(crate) fn matches_sboterm_pattern(value: &Option<String>) -> bool {
    let pattern = Regex::new(r"^SBO:\d{7}$").unwrap();
    matches_pattern(value, &pattern)
}

/// Checks that a given value conforms to the **XML 1.0 ID** syntax.
pub(crate) fn matches_xml_id_pattern(value: &Option<String>) -> bool {
    let pattern = formatcp!(
        "^[{0}_:][{0}{1}.\\-_:{2}{3}]*$",
        xml_definitions::build_letter_group(),
        xml_definitions::build_digit_group(),
        xml_definitions::build_combining_char_group(),
        xml_definitions::build_extender_group(),
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
    //      The `&` `'` and `"` escaping is probably handled by `biodivine-lib-xml-dom` and we should just see
    //      "normal", unescaped strings in XML attributes, hence this check is probably a bit
    //      too aggressive. But we should make sure to test this.
    let pattern =
        Regex::new(r###"^[^&'"\uFFFE\uFFFF]*(?:&(amp|apos|quot);[^&'"\uFFFE\uFFFF]*)*$"###)
            .unwrap();
    matches_pattern(value, &pattern)
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
        issues.push(SbmlIssue::new("10311", xml_element, message))
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
    unit_ref: Option<SId>,
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
        issues.push(SbmlIssue::new("10313", xml_element, message));
    }
}

#[cfg(test)]
mod version_tests {
    use std::collections::BTreeSet;

    use crate::Sbml;

    fn rules_of(document: &str) -> BTreeSet<String> {
        Sbml::read_str(document)
            .expect("the document is well-formed XML")
            .validate()
            .expect("validation cannot be cancelled")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect()
    }

    /// ### Rule 10202, the version difference
    /// The MathML subset of Level 3 Version 1 is the Version 2 subset without the five operators
    /// the specification marks with `\vertwo` (`implies`, `max`, `min`, `quotient`, `rem`), so a
    /// Version 1 document that uses one of them has an unknown `<math>` child (10202) and the same
    /// document as Version 2 does not.
    ///
    /// The two fixtures differ in the `version` attribute *and* in the namespace, because that is
    /// how a Version 1 document reaches this library: the test-suite harness lifts the core
    /// namespace of its `l3v1` documents to the Version 2 one and keeps `version="1"`, so the
    /// version attribute is what a check has to read (a document in the Version 1 namespace is not
    /// resolved by the object model at all).
    #[test]
    fn the_mathml_subset_of_a_version_1_document_excludes_the_version_2_operators() {
        let document_of_version = |version: &str| {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="{version}">
  <model>
    <listOfParameters><parameter id="p" constant="true"/></listOfParameters>
    <listOfConstraints>
      <constraint><math xmlns="http://www.w3.org/1998/Math/MathML">
        <apply><implies/><ci>p</ci><ci>p</ci></apply>
      </math></constraint>
    </listOfConstraints>
  </model>
</sbml>"#
            )
        };
        assert!(
            rules_of(&document_of_version("1")).contains("10202"),
            "`implies` is not in the Version 1 subset"
        );
        assert!(
            !rules_of(&document_of_version("2")).contains("10202"),
            "`implies` is in the Version 2 subset"
        );
    }
}

#[cfg(test)]
mod version_applicability_tests {
    use std::collections::BTreeSet;

    use crate::Sbml;

    fn rules_of(document: &str) -> BTreeSet<String> {
        Sbml::read_str(document)
            .expect("the document is well-formed XML")
            .validate()
            .expect("validation cannot be cancelled")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect()
    }

    /// ### Rules 10223, 10224, 10225 (new in Level 3 Version 2)
    /// The specification marks the three `rateOf` rules with `\validationRuleVerTwo`, and no rule
    /// of those numbers exists in the Version 1 specification: the `rateOf` csymbol itself is a
    /// Version 2 addition. A Version 1 document is therefore not checked against them, and the same
    /// document as Version 2 is.
    #[test]
    fn the_rate_of_rules_apply_to_a_version_2_document_only() {
        let document_of_version = |version: &str| {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="{version}">
  <model>
    <listOfParameters>
      <parameter id="p" constant="false"/>
      <parameter id="q" constant="false"/>
    </listOfParameters>
    <listOfRules>
      <rateRule variable="p"><math xmlns="http://www.w3.org/1998/Math/MathML">
        <apply><csymbol definitionURL="http://www.sbml.org/sbml/symbols/rateOf"/>
          <ci>p</ci><ci>q</ci></apply>
      </math></rateRule>
    </listOfRules>
  </model>
</sbml>"#
            )
        };
        let version_2 = rules_of(&document_of_version("2"));
        let version_1 = rules_of(&document_of_version("1"));
        assert!(
            version_2.contains("10223") || version_2.contains("10224"),
            "the Version 2 document is checked against the rateOf rules: {version_2:?}"
        );
        assert!(
            !version_1.contains("10223") && !version_1.contains("10224"),
            "the Version 1 document is not checked against the rateOf rules: {version_1:?}"
        );
    }
}
