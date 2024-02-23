use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, sanity_check, sanity_check_of_list,
    validate_list_of_objects, SanityCheckable, SbmlValidable,
};
use crate::core::{
    KineticLaw, LocalParameter, ModifierSpeciesReference, Reaction, SBase, SpeciesReference,
};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlList, XmlWrapper};
use crate::{SbmlIssue, SbmlIssueSeverity};
use std::collections::HashSet;

impl SbmlValidable for Reaction {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(
            Some(self.id().get()),
            self.xml_element(),
            issues,
            identifiers,
        );

        if let Some(list_of_reactants) = self.reactants().get() {
            validate_list_of_objects(&list_of_reactants, issues, identifiers);
        }
        if let Some(list_of_products) = self.products().get() {
            validate_list_of_objects(&list_of_products, issues, identifiers);
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            validate_list_of_objects(&list_of_modifiers, issues, identifiers);
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.validate(issues, identifiers);
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
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);
    }
}

impl SanityCheckable for SpeciesReference {}

impl SbmlValidable for ModifierSpeciesReference {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);
    }
}

impl SanityCheckable for ModifierSpeciesReference {}

impl SbmlValidable for KineticLaw {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            validate_list_of_objects(&list_of_local_parameters, issues, identifiers);
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
                issues.push(SbmlIssue {
                    element: local_parameter.raw_element(),
                    message: format!("The identifier ('{0}') of <localParameter> is already present in the <listOfLocalParameters>.",
                                     id),
                    rule: "10303".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            } else {
                identifiers.insert(id);
            }
        }
    }
}

impl SbmlValidable for LocalParameter {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, _identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues)
    }
}

impl SanityCheckable for LocalParameter {}
