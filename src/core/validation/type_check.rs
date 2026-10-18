use crate::cancellation::CancellationCheck;
use crate::constants::deltas;
use crate::constants::namespaces::{
    document_package_namespace_for, is_core_namespace, is_foreign_namespace,
    is_well_formed_sbml_namespace, namespace_of_prefix, package_of_namespace_url, written_url_of,
    NS_SBML_CORE, URL_MATHML, URL_PACKAGE_FBC, URL_PACKAGE_LAYOUT, URL_PACKAGE_QUAL,
};
use crate::constants::version::document_level_version_or_base;
use crate::constraint::FbcType;
use crate::core::SId;
use crate::qual::{Sign, TransitionInputEffect, TransitionOutputEffect};
use crate::xml::{
    OptionalSbmlProperty, SbmlObject, SbmlProperty, XmlElement, XmlList, XmlProperty,
    XmlPropertyType, XmlWrapper,
};
use crate::SbmlResult;
use crate::{Sbml, SbmlIssue};
use std::collections::{HashMap, HashSet};

/// Denotes an element that possess a way to self-test against the most critical checks. This
/// should be executed **before** the actual document validation. If the type check fails,
/// validation is likely to panic and crash the program.
///
/// Once a document is type checked, performing only safe operations on it should always keep
/// it in a type checked state.
pub(crate) trait CanTypeCheck: XmlWrapper {
    /// Type-check this object and the objects it contains.
    ///
    /// Every implementation accounts for itself with exactly one
    /// [`CancellationCheck::tick`](crate::cancellation::CancellationCheck::tick) before it starts
    /// checking (and then recurses into its children, each of which ticks for itself), so that the
    /// type check of a large document is cancellable like the rest of
    /// [`Sbml::validate`](crate::Sbml::validate).
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of
    /// the current thread has fired.
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        Ok(())
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

    // The version-aware tables: the element tables of this crate are the Level 3 Version 2 base
    // tables, and the deltas of the document's own level and version are applied here. The version
    // is resolved once per element (the table lookups below all use it).
    let version = document_level_version_or_base(xml_element);

    // Check that all required attributes are present.
    if let Some(required) = deltas::required_attributes(element_name.as_str(), version) {
        for req_attr in required.iter() {
            let (prefix, name) = separate_prefix_name(req_attr);
            let namespace = namespace_of_prefix(prefix);
            let property = SbmlProperty::<String>::new(xml_element, name, namespace, namespace);

            if namespace != NS_SBML_CORE {
                let package_declaration = Sbml::try_for_child(xml_element)
                    .unwrap()
                    .declared_package_prefix(namespace);
                if package_declaration.is_err() {
                    // This package is not declared, hence it's required attributes are not relevant.
                    continue;
                }
                // A package versions its own attributes: `model@fbc:strict` is required by FBC
                // Version 2, does not exist in Version 1, and was removed in Version 3. The
                // requirement below is the requirement of the version this crate models, so it is
                // imposed only on a document that *uses* that version; a document that declares
                // another published version of the same package is bound by that version's rules,
                // which are not the tables used here.
                if !uses_modelled_package_version(xml_element, namespace) {
                    continue;
                }
            }

            if !property.is_set() {
                let message = format!(
                    "Sanity check failed: missing required attribute [{req_attr}] on <{element_name}>."
                );
                let rule_id = tag_to_attribute_rule_id(element_name.as_str(), req_attr)
                    .unwrap_or("SANITY_CHECK");
                issues.push(SbmlIssue::new(rule_id, xml_element, message));
            }
        }
    }

    // Typecheck all relevant attributes.
    for attr in attributes.keys() {
        let attr_name = attr.local_name_str();
        let (_prefix, name) = separate_prefix_name(attr_name);
        let Some(types) = deltas::attribute_types(element_name.as_str(), version) else {
            break;
        };

        // t => (attribute name, attribute value)
        for (attr_id, attr_type) in types.iter() {
            let (_prefix, name2) = separate_prefix_name(attr_name);
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

/// The element-local findings of [`internal_type_check`], as a value instead of a report.
///
/// The walk of the object model asks this question before it reads an object's typed properties:
/// a type-check problem means the typed view of that object is unusable, so the walk skips the
/// object and reports that it did instead of panicking inside a property read. The probe runs the
/// very same checks as the type-check pass (into a report of its own, so it cannot add the same
/// finding to the caller's report twice) - a check that exists for one purpose exists for the other.
pub(crate) fn type_check_problems(xml_element: &XmlElement) -> Vec<SbmlIssue> {
    let mut probe = Vec::new();
    internal_type_check(xml_element, &mut probe);
    crate::layout::validation::curve::check_curve_segment_element(xml_element, &mut probe);
    probe
}

/// A utility function to perform a type check on an [XmlList] instance and all its valid
/// child elements.
///
/// The list container is a visited object of the walk as well, so it accounts for itself here and
/// then leaves the accounting of every child to the child's own
/// [`CanTypeCheck::type_check`].
///
/// # Errors
///
/// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) if a cancellation trigger of the
/// current thread has fired.
pub(crate) fn type_check_of_list<T: CanTypeCheck + SbmlObject>(
    xml_list: &XmlList<T>,
    issues: &mut Vec<SbmlIssue>,
    check: &mut CancellationCheck,
) -> SbmlResult<()> {
    check.tick()?;
    let element = xml_list.xml_element();
    let name = element.tag_name();
    issues.extend(type_check_problems(element));

    if let Some(allowed) =
        deltas::allowed_children(name.as_str(), document_level_version_or_base(element))
    {
        // Only sanity-check the allowed children. The rest is going to be reported as an
        // error and is probably malformed anyway.
        for object in xml_list.iter() {
            if !allowed.contains(&object.xml_element().tag_name().as_str()) {
                // This is a linear-time check, which is not great, but the list of allowed
                // children is typically very short anyway.
                continue;
            }
            if !T::matches_element(object.xml_element()) {
                // An SBase payload the container permits, or a stray element: it is not an item of
                // this list, so the item's own checks do not describe it (the container reports the
                // element it does not permit, and the payloads are checked by `SBase` itself).
                continue;
            }
            object.type_check(issues, check)?;
        }
    } else {
        // Sanity-check everyone.
        for object in xml_list.iter() {
            object.type_check(issues, check)?;
        }
    }

    Ok(())
}

/// Whether a namespace URL is one that no SBML specification defines, and whose content rule 10102
/// therefore forbids on an SBML element.
///
/// A namespace of the standard is accepted whatever package it belongs to (see
/// [`is_well_formed_sbml_namespace`]), and so is a foreign namespace whose content is outside the
/// scope of the rule (the XML Schema instance namespace, XHTML, MathML and RDF - the namespaces an
/// `annotation`, a `notes` object and an `xsi:type` attribute use). Everything else is an undefined
/// SBML namespace: the corpus exercises both shapes, a URL under the SBML tree that is not shaped
/// like a namespace of the standard (`http://www.sbml.org/sbml/version1`) and a URL that is not an
/// SBML URL at all (`http://anything`).
fn is_undefined_sbml_namespace(url: &str) -> bool {
    !is_well_formed_sbml_namespace(url) && !is_foreign_namespace(url)
}

/// The namespace an attribute of an element is in, if it has one.
///
/// The attribute is named the way the tables name it (`prefix:local` or `local`), so the name is
/// resolved against the namespace declarations that are in scope at the element - the same way the
/// typed property layer resolves it.
fn attribute_namespace(
    xml_element: &XmlElement,
    full_name: &str,
) -> Option<biodivine_lib_xml_dom::Namespace> {
    let (prefix, _local) = separate_prefix_name(full_name);
    if prefix.is_empty() {
        return None;
    }
    // The declarations in scope are reported as `(prefix, binding)` pairs, and the binding of a
    // prefix is what the attribute name resolves to.
    xml_element
        .element()
        .namespaces_in_scope()
        .into_iter()
        .find(|(declared, _binding)| declared.as_ref().map(|it| it.as_str()) == Some(prefix))
        .and_then(|(_prefix, binding)| binding)
}

/// Whether `namespace` is the namespace of a package *version* whose content model the crate's
/// tables describe.
///
/// The tables of this crate are the tables of one version of each package (the one its namespace
/// constants name), and a package's versions differ in their attributes, their children and their
/// requirements. A document that declares another published version of the same package therefore
/// keeps its content out of the content-model checks: what the tables would report about it would be
/// the *other* version's opinion.
///
/// See `validate_allowed_children` and the required-attribute loop for what each of the two
/// directions costs; both are measured in `notes/g42-report.md`.
fn is_package_of_the_modelled_version(url: &str) -> bool {
    url == URL_PACKAGE_FBC || url == URL_PACKAGE_LAYOUT || url == URL_PACKAGE_QUAL
}

/// Whether the document that owns `element` uses the package version the crate models, i.e. binds
/// one of the namespace URLs of that package whose name the tables use.
///
/// A document that declares no version of the package at all counts as using the modelled one: the
/// callers have already established that the package is declared (through
/// [`declared_package_prefix`](crate::Sbml::declared_package_prefix), which accepts every published
/// version), and a package bound under a prefix other than the conventional one is still the
/// package.
fn uses_modelled_package_version(
    xml_element: &XmlElement,
    namespace: crate::xml::Namespace,
) -> bool {
    let Some(prefix) = package_of_namespace_url(namespace.1) else {
        return true;
    };
    let Some(modelled) = written_url_of(prefix) else {
        return true;
    };
    document_package_namespace_for(xml_element.element(), (prefix, modelled)).1 == modelled
}
fn separate_prefix_name(name: &str) -> (&str, &str) {
    match name.split_once(':') {
        Some((prefix, local)) => (prefix, local),
        None => ("", name),
    }
}

/// Performs a type check of a value of a specific attribute.
/// If check fails, error is logged in *issues*.
fn type_check_of_property<T: XmlPropertyType>(
    attribute_name: &'static str,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    let (prefix, name) = separate_prefix_name(attribute_name);
    let namespace = namespace_of_prefix(prefix);
    let package_declaration = Sbml::try_for_child(xml_element)
        .unwrap()
        .declared_package_prefix(namespace);
    if package_declaration.is_err() {
        // This package is not declared, hence it's required attributes are not relevant.
        return;
    }
    if !uses_modelled_package_version(xml_element, namespace) {
        // The attribute's *type* is described by the version this crate models; a document that
        // declares another published version is not checked against it (see the required-attribute
        // loop for the measured reason).
        return;
    }
    let property = OptionalSbmlProperty::<T>::new(xml_element, name, namespace, namespace);
    if let Some(err) = property.get_checked().err() {
        // TODO:
        //  This also maps to a lot of concrete rule IDs based on the tag/attribute and
        //  will need a separate method to resolve.
        let message = format!(
            "Sanity check failed: {0} On the attribute [{1}].",
            err, attribute_name
        );
        issues.push(SbmlIssue::new("SANITY_CHECK", xml_element, message));
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
    let attributes = xml_element
        .attributes()
        .keys()
        .map(|key| key.to_string())
        .collect::<Vec<String>>();

    validate_allowed_attributes(xml_element, &attributes, issues);
    validate_allowed_children(xml_element, issues);
    validate_required_children(xml_element, issues);
    validate_unique_children(xml_element, issues);
}

/// Validates for a given element that its attributes (keys) are only from predefined set of
/// attributes (keys). If not, an error is logged in the vector of issues.
pub fn validate_allowed_attributes(
    xml_element: &XmlElement,
    attributes: &[String],
    issues: &mut Vec<SbmlIssue>,
) {
    let element_name = xml_element.tag_name();
    let version = document_level_version_or_base(xml_element);
    let Some(allowed_attributes) = deltas::allowed_attributes(element_name.as_str(), version)
    else {
        // This might happen if someone runs this method on a tag in the SBML namespace that
        // is not officially supported. In such a case, other functions should report that error,
        // but we just silently continue.
        return;
    };

    for full_name in attributes {
        let (prefix, attr_name) = separate_prefix_name(full_name);

        if !prefix.is_empty() {
            // The SBML core attributes live in no namespace, and an attribute a package adds lives
            // in that package's namespace, so an attribute with a prefix is not a core attribute
            // and is not checked against the core attribute table.
            //
            // It is checked for the other half of rule 10102 instead: an attribute in a namespace
            // that is neither a namespace of the standard nor one of the foreign namespaces an
            // `annotation` or `notes` payload uses is an attribute in an *undefined* SBML
            // namespace, and the rule forbids it. The report goes under the rule of the element
            // when the tables have one for it (a compartment's attributes are rule 20517), and
            // under 10102 otherwise.
            //
            // The attributes of the root are not checked here: they belong to whatever namespace
            // the root declares, and the namespace of the root itself is what rules 20101 and
            // 20102 are about. A root whose own namespace is foreign would otherwise turn every
            // one of its attributes into an "undefined SBML namespace" report.
            if element_name != "sbml" {
                if let Some(namespace) = attribute_namespace(xml_element, full_name) {
                    let url = namespace.uri();
                    if is_undefined_sbml_namespace(url) {
                        let message = format!(
                            "The attribute [{attr_name}] of the element <{element_name}> is in \
                             the namespace '{url}', which no SBML specification defines."
                        );
                        // The element's own attribute rule when the tables have one (an attribute
                        // of a compartment is rule 20517), and the general rule of an undefined
                        // element or attribute otherwise.
                        let rule_id = tag_to_attribute_rule_id(element_name.as_str(), attr_name)
                            .unwrap_or("10102");
                        issues.push(SbmlIssue::new(rule_id, xml_element, message));
                    }
                }
            }
            continue;
        }

        if !allowed_attributes.contains(&attr_name) {
            let message = format!(
                "An unknown attribute [{}] of the element <{}> found.",
                attr_name, element_name
            );

            let rule_id =
                tag_to_attribute_rule_id(element_name.as_str(), attr_name).unwrap_or("10102");
            issues.push(SbmlIssue::new(rule_id, xml_element, message));
        }
    }
}

/// Validates for a given element that its children (tag names) are only from the predefined set
/// of children (tag names). If not, an error is logged in the vector of issues.
pub(crate) fn validate_allowed_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    let version = document_level_version_or_base(xml_element);
    let Some(allowed_children) = deltas::allowed_children(element_name.as_str(), version) else {
        // If the element has no allowed children registered, then it can be safely skipped
        // (we have no relevant rules for this element and anything is thus allowed)
        return;
    };

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
            issues.push(SbmlIssue::new("10201", xml_element, message));
        } else if is_undefined_sbml_namespace(&child_namespace)
            && is_well_formed_sbml_namespace(xml_element.namespace_url().as_str())
        {
            // The other half of rule 10102: a child element in a namespace that is neither a
            // namespace of the standard nor a foreign one is an element in an *undefined* SBML
            // namespace. A child of a package the crate does not model is not reported here (the
            // namespace is a namespace of the standard; what it may contain is that package's
            // business), and neither is a foreign element (the payload of an `annotation` or of a
            // `notes` object).
            let message = format!(
                "An unknown child <{child_name}> of the element <{element_name}> found: the \
                 namespace '{child_namespace}' is not one an SBML specification defines."
            );
            let rule_id = tag_to_allowed_child_rule_id(element_name.as_str()).unwrap_or("10102");
            issues.push(SbmlIssue::new(rule_id, xml_element, message));
        } else if is_core_namespace(child_namespace.as_str())
            || is_package_of_the_modelled_version(child_namespace.as_str())
        {
            // All other children are expected to be in the core namespace *of the document's own
            // level and version* - a Level 3 Version 1 document's children are in the Level 3
            // Version 1 core namespace, not in the one this crate writes with - or in the package
            // namespace whose content model the tables describe. The second test is deliberately
            // *not* `is_modelled_package_namespace`: the tables are the tables of the version this
            // crate models (`FBC_NAMESPACE_URLS[1]`, layout and qual Version 1), and a document that
            // declares another published version of the same package keeps its content there, so
            // applying this version's content model to it would report the other version's legal
            // elements as unknown (FBC Version 1's `listOfFluxBounds` is the measured example, 81
            // events, and the coarse fix was a regression). Anything else is skipped.
            if !allowed_children.contains(&child_name.as_str()) {
                let message = format!(
                    "An unknown child <{}> of the element <{}> found.",
                    child_name, element_name
                );
                let rule_id =
                    tag_to_allowed_child_rule_id(element_name.as_str()).unwrap_or("10102");
                issues.push(SbmlIssue::new(rule_id, xml_element, message));
            }
        }
    }
}

pub(crate) fn validate_required_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    // The *base* table only: a child it requires is one the typed view reads as a required child,
    // so a missing one is a reason not to walk the element (the walk asks for this check). The
    // requirements a version adds on top are rules and are reported by
    // [`validate_version_required_children`] instead, which the walk calls without making the
    // report suppress the object.
    let required_children = deltas::base_required_children(element_name.as_str());

    report_missing_children(
        xml_element,
        element_name.as_str(),
        required_children,
        issues,
    );
}

/// Reports the required children the document's own version adds to the base table.
///
/// These are rules of the version, not properties of the typed view: the crate's accessors for them
/// are optional children and lists, so a document that omits one is still walked - its own rules
/// and the rules of everything below it are reported as usual. Reporting them from the type check
/// instead would turn each missing `math` into a `not-validated` for the whole subtree, which loses
/// the very reports the version's other rules are about.
pub(crate) fn validate_version_required_children(
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    let element_name = xml_element.tag_name();
    let version = document_level_version_or_base(xml_element);
    let added = deltas::added_required_children(element_name.as_str(), version);
    report_missing_children(xml_element, element_name.as_str(), added, issues);
}

/// One report per missing child of `required`, under the rule the element's tag maps to.
fn report_missing_children(
    xml_element: &XmlElement,
    element_name: &str,
    required: &[&'static str],
    issues: &mut Vec<SbmlIssue>,
) {
    let mut child_tag_names = HashSet::new();
    for child in xml_element.child_elements() {
        let child_name = child.tag_name();
        child_tag_names.insert(child_name);
    }

    for required in required.iter() {
        if !child_tag_names.contains(*required) {
            let message =
                format!("Missing required child <{required}> of the element <{element_name}>.");
            // TODO: Check if these rule IDs are ok.
            let rule_id = tag_to_required_child_rule_id(element_name).unwrap_or("10102");
            issues.push(SbmlIssue::new(rule_id, xml_element, message));
        }
    }
}

/// Validates for a given element that its children that are required to appear at most once
/// indeed do. Logs error if this is violated.
pub(crate) fn validate_unique_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    let element_name = xml_element.tag_name();
    let version = document_level_version_or_base(xml_element);
    let Some(unique_children) = deltas::unique_children(element_name.as_str(), version) else {
        // No rules are given for this child element.
        return;
    };

    let mut counts = HashMap::new();
    for child in xml_element.child_elements() {
        let child_name = child.tag_name();
        let child_namespace = child.namespace_url();
        if is_core_namespace(child_namespace.as_str())
            || child_namespace == URL_MATHML
            || child_namespace == URL_PACKAGE_LAYOUT
            || child_namespace == URL_PACKAGE_FBC
            || child_namespace == URL_PACKAGE_QUAL
        {
            // Core children of *any* modelled level/version take part, not only the ones in the
            // namespace this crate writes with, plus the MathML namespace and the package
            // namespaces whose content model the tables describe (see the comment in
            // `validate_allowed_children` for why the modelled *version* is the test).
            let entry = counts.entry(child_name);
            let count = entry.or_insert(0usize);
            *count += 1;
        }
    }

    for name in unique_children.iter() {
        if let Some(count) = counts.get(*name) {
            if *count > 1 {
                let message = format!(
                    "Multiple instances of child <{}> found in element <{}>.",
                    name, element_name
                );
                let rule_id =
                    tag_to_unique_child_rule_id(element_name.as_str(), name).unwrap_or("10102");
                issues.push(SbmlIssue::new(rule_id, xml_element, message));
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
        "geneProductAssociation" => Some("fbc-20803"),
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
        // layout package
        "dimensions" => Some("layout-21703"),
        "curve" => Some("layout-21402"),
        "boundingBox" => Some("layout-21302"),
        "position" => Some("layout-21204"),
        "referenceGlyph" => Some("layout-21104"),
        "speciesReferenceGlyph" => Some("layout-21004"),
        "textGlyph" => Some("layout-20904"),
        "listOfSubGlyphs" => Some("layout-20813"),
        "listOfReferenceGlyphs" => Some("layout-20811"),
        "generalGlyph" => Some("layout-20804"),
        "listOfSpeciesReferenceGlyphs" => Some("layout-20711"),
        "reactionGlyph" => Some("layout-20704"),
        "speciesGlyph" => Some("layout-20604"),
        "compartmentGlyph" => Some("layout-20504"),
        "graphicalObject" => Some("layout-20404"),
        "listOfTextGlyphs" => Some("layout-20316"),
        "listOfAdditionalGraphicalObjects" => Some("layout-20313"),
        "listOfReactionGlyphs" => Some("layout-20311"),
        "listOfSpeciesGlyphs" => Some("layout-20309"),
        "listOfCompartmentGlyphs" => Some("layout-20307"),
        "layout" => Some("layout-20305"),
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
        //fbc  package
        "listOfFluxObjectives" => Some("fbc-20508"),
        "geneProductAssociation" => Some("fbc-20805"),
        //qual package
        "transition" => Some("20406"),
        "listOfInputs" => Some("qual-20407"),
        "listOfOutputs" => Some("qual-20408"),
        "listOfFunctionTerms" => Some("qual-20409"),
        "functionTerm" => Some("qual-20804"),
        //layout package
        "boundingBox" => Some("layout-21303"),
        "layout" => Some("layout-20302"),
        "listOfCompartmentGlyphs" => Some("layout-20308"),
        "listOfSpeciesGlyphs" => Some("layout-20310"),
        "listOfReactionGlyphs" => Some("layout-20312"),
        "listOfAdditionalGraphicalObjects" => Some("layout-20304"),
        "listOfTextGlyphs" => Some("layout-20317"),
        "listOfSpeciesReferenceGlyphs" => Some("layout-20710"),
        "listOfReferenceGlyphs" => Some("layout-20810"),
        "listOfSubGlyphs" => Some("layout-20812"),
        "listOfCurveSegments" => Some("layout-21406"),
        _ => None,
    }
}

fn tag_to_required_child_rule_id(tag_name: &str) -> Option<&'static str> {
    match tag_name {
        // The core requirements that differ between the two versions of Level 3: each of these
        // elements has exactly one required child in Version 1 and no required child in Version 2,
        // so the tag name alone identifies the rule.
        "sbml" => Some("20201"),
        "functionDefinition" => Some("20306"),
        "initialAssignment" => Some("20804"),
        "assignmentRule" | "rateRule" | "algebraicRule" => Some("20907"),
        "constraint" => Some("21007"),
        "kineticLaw" => Some("21130"),
        "trigger" => Some("21209"),
        "delay" => Some("21210"),
        "eventAssignment" => Some("21213"),
        "priority" => Some("21231"),
        "event" => Some("21201"),
        "layout" => Some("layout-20315"),
        "graphicalObject" => Some("layout-20407"),
        "textGlyph" => Some("layout-20903"),
        "boundingBox" => Some("layout-21303"),
        "curve" => Some("layout-21403"),
        "listOfFunctionTerms" => Some("qual-20409"),
        // The required children of a curve segment are not in this table: the two variants require
        // different points, and only the discriminator says which variant an element is. The check
        // in `layout::validation::curve` owns them and reports the variant's rule id
        // (`layout-21503` or `layout-21603`).
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
        // The core lists of a `model` are all covered by 20205 ("There may be at most one instance
        // of each of the following kind of object in a Model object: ListOfFunctionDefinitions,
        // ..."), but a *package* list has a rule of its own in that package: `fbc-20201` for the
        // Flux Balance Constraints lists, `layout-20201` for the Layout list and `qual-20201` for
        // the Qualitative Models lists. Reporting the core id for those would be reporting the
        // wrong rule for the object that is duplicated.
        ("model", "listOfLayouts") => Some("layout-20201"),
        ("model", "listOfGeneProducts")
        | ("model", "listOfObjectives")
        | ("model", "listOfUserDefinedConstraints") => Some("fbc-20201"),
        ("model", "listOfTransitions") | ("model", "listOfQualitativeSpecies") => {
            Some("qual-20201")
        }
        ("model", _) => Some("20205"),
        ("functionDefinition", "math") => Some("20306"),
        ("unitDefinition", "listOfUnits") => Some("20414"),
        ("initialAssignment", "math") => Some("20804"),
        ("assignmentRule", "math") | ("rateRule", "math") | ("algebraicRule", "math") => {
            Some("20907")
        }
        ("constraint", "math") => Some("21007"),
        ("constraint", "message") => Some("21008"),
        // Same as `model` above, there is a lot of unique children to enumerate, and the Flux
        // Balance Constraints package gives the `geneProductAssociation` a rule of its own
        // (`fbc-20701`: "A Reaction object may contain one and only one instance of the
        // GeneProductAssociation element").
        ("reaction", "geneProductAssociation") => Some("fbc-20701"),
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
        ("transition", "listOfFunctionTerms")
        | ("transition", "listOfInputs")
        | ("transition", "listOfOutputs") => Some("qual-20405"),
        ("listOfFunctionTerms", "defaultTerm") => Some("qual-20409"),
        // A function term carries exactly one formula, and the rule that says so is the same one
        // that requires it in the first place.
        ("functionTerm", "math") => Some("qual-20804"),
        ("layout", _) => Some("layout-20303"),
        ("graphicalObject", "boundingBox") => Some("layout-20403"),
        ("compartmentGlyph", "boundingBox") => Some("layout-20503"),
        ("speciesGlyph", "boundingBox") => Some("layout-20603"),
        ("reactionGlyph", "boundingBox") => Some("layout-20703"),
        ("reactionGlyph", "curve") => Some("layout-20703"),
        ("reactionGlyph", "listOfSpeciesReferenceGlyphs") => Some("layout-20703"),
        ("generalGlyph", "boundingBox") => Some("layout-20803"),
        ("generalGlyph", "curve") => Some("layout-20803"),
        ("generalGlyph", "listOfReferenceGlyphs") => Some("layout-20803"),
        ("generalGlyph", "listOfSubGlyphs") => Some("layout-20803"),
        ("textGlyph", "boundingBox") => Some("layout-20903"),
        ("speciesReferenceGlyph", "boundingBox") => Some("layout-21003"),
        ("speciesReferenceGlyph", "curve") => Some("layout-21003"),
        ("referenceGlyph", "boundingBox") => Some("layout-21103"),
        ("referenceGlyph", "curve") => Some("layout-21103"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::Sbml;
    use std::collections::BTreeSet;

    fn rules_of(document: &str) -> BTreeSet<String> {
        Sbml::read_str(document)
            .expect("the document is well-formed XML")
            .validate()
            .expect("validation cannot be cancelled")
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect()
    }

    /// ### Rule 10102
    /// An SBML document must not contain undefined elements or attributes in the SBML Level 3 Core
    /// namespace or in a SBML Level 3 package namespace.
    ///
    /// The corpus exercises two shapes of an *undefined* SBML namespace: a URL under the SBML tree
    /// that is not shaped like a namespace of the standard, and a URL that is not an SBML URL at
    /// all. Both are reported, the first under the rule of the element the attribute belongs to
    /// (rule 20517 for a compartment) and the second as the general 10102.
    #[test]
    fn an_attribute_in_an_undefined_sbml_namespace_is_reported() {
        let document = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2"
      xmlns:appear="http://www.sbml.org/sbml/version1">
  <model>
    <listOfCompartments>
      <compartment id="c" constant="true" appear:shape="ellipse"/>
    </listOfCompartments>
  </model>
</sbml>"#;
        let rules = rules_of(document);
        assert!(rules.contains("20517"), "expected 20517 in {rules:?}");
    }

    /// An element in an undefined SBML namespace is the same rule, and a *package* namespace the
    /// crate does not model is not reported: the package's own rules are about its content.
    #[test]
    fn a_child_element_in_an_undefined_sbml_namespace_is_reported() {
        let undefined = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2"
      xmlns:appear="http://anything">
  <model>
    <appear:listOfShapes/>
  </model>
</sbml>"#;
        assert!(
            rules_of(undefined).contains("10102"),
            "an element in a namespace no specification defines is a 10102"
        );

        let known_package = r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2"
      xmlns:fbc="http://www.sbml.org/sbml/level3/version1/fbc/version2" fbc:required="false">
  <model>
    <listOfCompartments>
      <compartment id="c" constant="true"/>
    </listOfCompartments>
  </model>
</sbml>"#;
        let rules = rules_of(known_package);
        assert!(
            !rules.contains("10102"),
            "a well-formed package namespace is not an undefined namespace: {rules:?}"
        );
    }
}
