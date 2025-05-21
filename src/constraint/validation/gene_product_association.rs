use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for GeneProductAssociation {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(gene_product) = self.gene_product_ref().get() {
            gene_product.validate(issues, identifiers, meta_ids);
        }

        if let Some(and) = self.and().get() {
            validate_list_of_objects(&and, issues, identifiers, meta_ids);
        }

        if let Some(or) = self.or().get() {
            validate_list_of_objects(&or, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for GeneProductAssociation {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if (self.gene_product_ref().is_set() && self.and().is_set())
            || (self.gene_product_ref().is_set() && self.or().is_set())
            || (self.and().is_set() && self.or().is_set())
        {
            let message = "Only one of GeneProductRef, Or, And elements can be set.";
            issues.push(SbmlIssue::new_error(
                "fbc-20805",
                self.xml_element(),
                message,
            ));
            return;
        }

        if let Some(gene_product) = self.gene_product_ref().get() {
            internal_type_check(self.xml_element(), issues);
            gene_product.type_check(issues)
        }

        if let Some(and) = self.and().get() {
            if and.len() < 2 {
                let message = "And object must have at least two concrete Association objects.";
                issues.push(SbmlIssue::new_error("fbc-21003", self, message))
            } else {
                type_check_of_list(&and, issues)
            }
        }

        if let Some(or) = self.or().get() {
            if or.len() < 2 {
                let message = "And object must have at least two concrete Association objects.";
                issues.push(SbmlIssue::new_error("fbc-21103", self, message))
            } else {
                type_check_of_list(&or, issues)
            }
        }
    }
}
