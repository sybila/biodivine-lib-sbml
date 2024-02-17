use crate::core::validation::{
    apply_rule_10102, apply_rule_10301, validate_list_of_objects, SbmlValidable,
};
use crate::core::{
    KineticLaw, LocalParameter, ModifierSpeciesReference, Reaction, SBase, SpeciesReference,
};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlWrapper};
use crate::SbmlIssue;
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

impl SbmlValidable for SpeciesReference {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);
    }
}

impl SbmlValidable for ModifierSpeciesReference {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);
    }
}

impl SbmlValidable for KineticLaw {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues);
        apply_rule_10301(self.id().get(), self.xml_element(), issues, identifiers);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            validate_list_of_objects(&list_of_local_parameters, issues, identifiers);
        }

        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl SbmlValidable for LocalParameter {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, _identifiers: &mut HashSet<String>) {
        apply_rule_10102(self.xml_element(), issues)
    }
}
