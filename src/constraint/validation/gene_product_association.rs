use crate::cancellation::CancellationCheck;
use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::xml::{OptionalXmlChild, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for GeneProductAssociation {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        if let Some(gene_product) = self.gene_product_ref().get() {
            gene_product.validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(and) = self.and().get() {
            validate_list_of_objects(&and, issues, identifiers, meta_ids, check)?;
        }

        if let Some(or) = self.or().get() {
            validate_list_of_objects(&or, issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for GeneProductAssociation {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if (self.gene_product_ref().is_set() && self.and().is_set())
            || (self.gene_product_ref().is_set() && self.or().is_set())
            || (self.and().is_set() && self.or().is_set())
        {
            let message = "Only one of GeneProductRef, Or, And elements can be set.";
            issues.push(SbmlIssue::new("fbc-20805", self.xml_element(), message));
            return Ok(());
        }

        if let Some(gene_product) = self.gene_product_ref().get() {
            issues.extend(type_check_problems(self.xml_element()));
            gene_product.type_check(issues, check)?
        }

        if let Some(and) = self.and().get() {
            if and.len() < 2 {
                let message = "And object must have at least two concrete Association objects.";
                issues.push(SbmlIssue::new("fbc-21003", self, message))
            } else {
                type_check_of_list(&and, issues, check)?
            }
        }

        if let Some(or) = self.or().get() {
            if or.len() < 2 {
                let message = "And object must have at least two concrete Association objects.";
                issues.push(SbmlIssue::new("fbc-21103", self, message))
            } else {
                type_check_of_list(&or, issues, check)?
            }
        }
        Ok(())
    }
}
