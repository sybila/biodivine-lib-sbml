use crate::core::validation::apply_rule_10102;
use crate::core::{
    KineticLaw, LocalParameter, ModifierSpeciesReference, Reaction, SpeciesReference,
};
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;

impl Reaction {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if self.reactants().is_set() {
            self.validate_list_of_reactants(issues);
        }
        if self.products().is_set() {
            self.validate_list_of_products(issues);
        }
        if self.modifiers().is_set() {
            self.validate_list_of_modifiers(issues);
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.validate(issues);
        }
    }

    fn validate_list_of_modifiers(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.modifiers().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let modifier = list.get(i);
            modifier.validate(issues);
        }
    }

    fn validate_list_of_products(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.products().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let product = list.get(i);
            product.validate(issues);
        }
    }

    fn validate_list_of_reactants(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.reactants().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let reactant = list.get(i);
            reactant.validate(issues);
        }
    }
}

impl SpeciesReference {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);
    }
}

impl ModifierSpeciesReference {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);
    }
}

impl KineticLaw {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);

        if self.local_parameters().is_set() {
            self.validate_list_of_local_parameters(issues);
        }
    }

    fn validate_list_of_local_parameters(&self, issues: &mut Vec<SbmlIssue>) {
        let list = self.local_parameters().get().unwrap();
        apply_rule_10102(list.xml_element(), issues);

        for i in 0..list.len() {
            let local_param = list.get(i);
            local_param.validate(issues);
        }
    }
}

impl LocalParameter {
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml_element(), issues);
    }
}
