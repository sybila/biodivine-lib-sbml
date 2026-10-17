use crate::cancellation::CancellationCheck;
use crate::constraint::association::{And, Association, GeneProductRef, Or};
use crate::constraint::GeneProduct;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::xml::{RequiredXmlProperty, XmlSubtype, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Association {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(association) = GeneProductRef::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(association) = And::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(association) = Or::try_cast_from_super(self) {
            association.validate(issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Association {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(association) = GeneProductRef::try_cast_from_super(self) {
            association.type_check(issues, check)?;
        }

        if let Some(association) = And::try_cast_from_super(self) {
            association.type_check(issues, check)?;
        }

        if let Some(association) = Or::try_cast_from_super(self) {
            association.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for GeneProductRef {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        apply_rule_fbc_20908(self, issues, self.gene_product().get());
        Ok(())
    }
}

impl CanTypeCheck for GeneProductRef {}

impl SbmlValidable for And {
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

impl CanTypeCheck for And {}

impl SbmlValidable for Or {
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

impl CanTypeCheck for Or {}

pub fn apply_rule_fbc_20908(element: &GeneProductRef, issues: &mut Vec<SbmlIssue>, sid_ref: SId) {
    let found = element.find_by_sid::<GeneProduct>(&sid_ref);

    if found.is_none() || found.unwrap().xml_element().tag_name() != "geneProduct" {
        let message = "Attribute [geneProduct] does not refer to an existing GeneProduct element!"
            .to_string();
        issues.push(SbmlIssue::new("fbc-20908", element, message))
    }
}
