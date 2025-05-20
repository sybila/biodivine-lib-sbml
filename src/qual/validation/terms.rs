use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::qual::terms::{AbstractTerm, DefaultTerm, FunctionTerm};
use crate::xml::{RequiredXmlChild, XmlSubtype};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for AbstractTerm {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.validate(issues, identifiers, meta_ids);
        } else if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.validate(issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for AbstractTerm {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.type_check(issues);
        } else if let Some(term) = DefaultTerm::try_cast_from_super(self) {
            term.type_check(issues);
        }
    }
}

impl SbmlValidable for DefaultTerm {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for DefaultTerm {}

impl SbmlValidable for FunctionTerm {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        _identifiers: &mut HashSet<SId>,
        _meta_ids: &mut HashSet<MetaId>,
    ) {
        self.math().get().validate(issues);
    }
}

impl CanTypeCheck for FunctionTerm {}
