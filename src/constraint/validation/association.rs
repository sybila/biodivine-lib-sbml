use crate::constraint::association::{And, Association, GeneProductRef, Or};
use crate::constraint::GeneProduct;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::{RequiredXmlChild, RequiredXmlProperty, XmlSubtype};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Association {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        if let Some(association) = GeneProductRef::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids);
        }

        if let Some(association) = And::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids);
        }

        if let Some(association) = Or::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for Association {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if let Some(association) = GeneProductRef::try_cast_from_super(self) {
            association.type_check(issues);
        }

        if let Some(association) = And::try_cast_from_super(self) {
            association.type_check(issues);
        }

        if let Some(association) = Or::try_cast_from_super(self) {
            association.type_check(issues);
        }
    }
}

impl SbmlValidable for GeneProductRef {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        apply_rule_fbc_20908(self, issues, self.gene_product().get());
    }
}

impl CanTypeCheck for GeneProductRef {}

impl SbmlValidable for And {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let lst = self.and().get();

        if lst.len() < 2 {
            let message = "And object must have at least two concrete Association objects.";
            issues.push(SbmlIssue::new_error("fbc-21003", self, message))
        }

        validate_list_of_objects(&lst, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for And {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        type_check_of_list(&self.and().get(), issues)
    }
}

impl SbmlValidable for Or {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let lst = self.or().get();

        if lst.len() < 2 {
            let message = "And object must have at least two concrete Association objects.";
            issues.push(SbmlIssue::new_error("fbc-21103", self, message))
        }

        validate_list_of_objects(&lst, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for Or {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        type_check_of_list(&self.or().get(), issues)
    }
}

pub fn apply_rule_fbc_20908(element: &GeneProductRef, issues: &mut Vec<SbmlIssue>, sid_ref: SId) {
    let found = element.find_by_sid::<GeneProduct>(&sid_ref);

    if found.is_none() {
        let message = "Attribute [geneProduct] does not refer to an existing GeneProduct element!"
            .to_string();
        issues.push(SbmlIssue::new_error("fbc:20908", element, message))
    }
}
