use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309,
    apply_rule_10310, apply_rule_10311, apply_rule_10312, apply_rule_10313, apply_rule_10401,
    sanity_check, sanity_check_of_list, validate_list_of_objects, SanityCheckable, SbmlValidable,
};
use crate::core::{
    KineticLaw, LocalParameter, ModifierSpeciesReference, Reaction, SBase, SpeciesReference,
};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlList, XmlProperty, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Reaction {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(Some(id.get()), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(Some(id.get()), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
        }
        if let Some(list_of_reactants) = self.reactants().get() {
            validate_list_of_objects(&list_of_reactants, issues, identifiers, meta_ids);
        }
        if let Some(list_of_products) = self.products().get() {
            validate_list_of_objects(&list_of_products, issues, identifiers, meta_ids);
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            validate_list_of_objects(&list_of_modifiers, issues, identifiers, meta_ids);
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.validate(issues, identifiers, meta_ids);
        }
    }
}

impl SanityCheckable for Reaction {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);

        if let Some(list_of_reactants) = self.reactants().get() {
            sanity_check_of_list(&list_of_reactants, issues);
        }
        if let Some(list_of_products) = self.products().get() {
            sanity_check_of_list(&list_of_products, issues);
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            sanity_check_of_list(&list_of_modifiers, issues);
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.sanity_check(issues);
        }
    }
}

impl SbmlValidable for SpeciesReference {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
        }
    }
}

impl SanityCheckable for SpeciesReference {}

impl SbmlValidable for ModifierSpeciesReference {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
        }
    }
}

impl SanityCheckable for ModifierSpeciesReference {}

impl SbmlValidable for KineticLaw {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10102(xml_element, issues);
        apply_rule_10301(id.get(), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(id.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
        }
        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            validate_list_of_objects(&list_of_local_parameters, issues, identifiers, meta_ids);
            KineticLaw::apply_rule_10303(&list_of_local_parameters, issues);
        }
        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SanityCheckable for KineticLaw {
    fn sanity_check(&self, issues: &mut Vec<SbmlIssue>) {
        sanity_check(self.xml_element(), issues);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            sanity_check_of_list(&list_of_local_parameters, issues);
        }
    }
}

impl KineticLaw {
    /// ### Rule 10303
    /// The value of the attribute id of every [LocalParameter] object defined within a [KineticLaw]
    /// object must be unique across the set of all such parameter definitions within that
    /// particular [KineticLaw] instance.
    pub(crate) fn apply_rule_10303(
        list_of_local_parameters: &XmlList<LocalParameter>,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let mut identifiers: HashSet<String> = HashSet::new();

        for local_parameter in list_of_local_parameters.as_vec() {
            let id = local_parameter.id().get();
            if identifiers.contains(&id) {
                let message = format!(
                    "The identifier ('{id}') of <localParameter> is \
                already present in the <listOfLocalParameters>."
                );
                issues.push(SbmlIssue::new_error("10303", &local_parameter, message));
            } else {
                identifiers.insert(id);
            }
        }
    }
}

impl SbmlValidable for LocalParameter {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        _identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();
        let units = self.units();

        apply_rule_10102(xml_element, issues);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(Some(id.get()), xml_element, issues);
        apply_rule_10311(units.name(), units.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);
        apply_rule_10313(units.name(), units.get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
        }
    }
}

impl SanityCheckable for LocalParameter {}
