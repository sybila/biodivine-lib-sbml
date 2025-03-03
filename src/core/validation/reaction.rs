use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10311, apply_rule_10313, validate_list_of_objects, SbmlValidable,
};
use crate::core::{
    KineticLaw, LocalParameter, MetaId, ModifierSpeciesReference, Reaction, SId, SpeciesReference,
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
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
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

impl CanTypeCheck for Reaction {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if let Some(list_of_reactants) = self.reactants().get() {
            type_check_of_list(&list_of_reactants, issues);
        }
        if let Some(list_of_products) = self.products().get() {
            type_check_of_list(&list_of_products, issues);
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            type_check_of_list(&list_of_modifiers, issues);
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.type_check(issues);
        }
    }
}

impl SbmlValidable for SpeciesReference {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for SpeciesReference {}

impl SbmlValidable for ModifierSpeciesReference {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for ModifierSpeciesReference {}

impl SbmlValidable for KineticLaw {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            validate_list_of_objects(&list_of_local_parameters, issues, identifiers, meta_ids);
            KineticLaw::apply_rule_10303(&list_of_local_parameters, issues);
        }
        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl CanTypeCheck for KineticLaw {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            type_check_of_list(&list_of_local_parameters, issues);
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
        let mut identifiers: HashSet<SId> = HashSet::new();

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
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let units = self.units();

        apply_rule_10311(units.name(), units.get_raw(), xml_element, issues);
        apply_rule_10313(units.name(), units.get(), xml_element, issues);
    }
}

impl CanTypeCheck for LocalParameter {}
