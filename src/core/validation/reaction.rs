use crate::cancellation::CancellationCheck;
use crate::constants::namespaces::NS_FBC;
use crate::constraint::FbcReaction;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10311, apply_rule_10313, validate_list_of_objects, SbmlValidable,
};
use crate::core::{
    KineticLaw, LocalParameter, MetaId, ModifierSpeciesReference, Reaction, SBase, SId,
    SpeciesReference,
};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, XmlList, XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Reaction {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(list_of_reactants) = self.reactants().get() {
            validate_list_of_objects(&list_of_reactants, issues, identifiers, meta_ids, check)?;
        }
        if let Some(list_of_products) = self.products().get() {
            validate_list_of_objects(&list_of_products, issues, identifiers, meta_ids, check)?;
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            validate_list_of_objects(&list_of_modifiers, issues, identifiers, meta_ids, check)?;
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.validate(issues, identifiers, meta_ids, check)?;
        }
        if self.sbml_root().declares_modelled_package(NS_FBC) {
            if let Some(fbc_reaction) = FbcReaction::try_cast_from_super(self) {
                fbc_reaction.validate(issues, identifiers, meta_ids, check)?;
            }
        }
        Ok(())
    }
}

impl CanTypeCheck for Reaction {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));

        if let Some(list_of_reactants) = self.reactants().get() {
            type_check_of_list(&list_of_reactants, issues, check)?;
        }
        if let Some(list_of_products) = self.products().get() {
            type_check_of_list(&list_of_products, issues, check)?;
        }
        if let Some(list_of_modifiers) = self.modifiers().get() {
            type_check_of_list(&list_of_modifiers, issues, check)?;
        }
        if let Some(kinetic_law) = self.kinetic_law().get() {
            kinetic_law.type_check(issues, check)?;
        }
        if self.sbml_root().declares_modelled_package(NS_FBC) {
            if let Some(fbc_reaction) = FbcReaction::try_cast_from_super(self) {
                fbc_reaction.type_check(issues, check)?;
            }
        }
        Ok(())
    }
}

impl SbmlValidable for SpeciesReference {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        Ok(())
    }
}

impl CanTypeCheck for SpeciesReference {}

impl SbmlValidable for ModifierSpeciesReference {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        Ok(())
    }
}

impl CanTypeCheck for ModifierSpeciesReference {}

impl SbmlValidable for KineticLaw {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            validate_list_of_objects(
                &list_of_local_parameters,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
            KineticLaw::apply_rule_10303(&list_of_local_parameters, issues);
        }
        if let Some(math) = self.math().get() {
            math.validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for KineticLaw {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));

        if let Some(list_of_local_parameters) = self.local_parameters().get() {
            type_check_of_list(&list_of_local_parameters, issues, check)?;
        }
        Ok(())
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
            // A missing or unparsable `id` is reported by the type check of the element itself, so
            // the aggregation skips it instead of reading it as a required value.
            let Some(id) = XmlProperty::get_checked(&local_parameter.id())
                .ok()
                .flatten()
            else {
                continue;
            };
            if identifiers.contains(&id) {
                let message = format!(
                    "The identifier ('{id}') of <localParameter> is \
                already present in the <listOfLocalParameters>."
                );
                issues.push(SbmlIssue::new("10303", &local_parameter, message));
            } else {
                identifiers.insert(id);
            }
        }
    }
}

impl SbmlValidable for LocalParameter {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let units = self.units();

        apply_rule_10311(units.simple_name(), units.get_raw(), xml_element, issues);
        apply_rule_10313(units.simple_name(), units.get(), xml_element, issues);
        Ok(())
    }
}

impl CanTypeCheck for LocalParameter {}
