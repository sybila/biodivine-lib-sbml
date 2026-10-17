use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::qual::terms::{AbstractTerm, DefaultTerm, FunctionTerm};
use crate::xml::{RequiredXmlChild, XmlChild, XmlSubtype};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for AbstractTerm {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.validate(issues, identifiers, meta_ids, check)?;
        } else if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.validate(issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for AbstractTerm {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.type_check(issues, check)?;
        } else if let Some(term) = FunctionTerm::try_cast_from_super(self) {
            term.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for DefaultTerm {
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

impl CanTypeCheck for DefaultTerm {}

impl SbmlValidable for FunctionTerm {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        _identifiers: &mut HashSet<SId>,
        _meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if self.math().get_raw().is_some() {
            self.math().get().validate(issues, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for FunctionTerm {}
