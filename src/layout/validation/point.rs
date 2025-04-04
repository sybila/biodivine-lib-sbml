use std::collections::HashSet;
use crate::core::{MetaId, SId};
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::SbmlValidable;
use crate::core::validation::type_check::CanTypeCheck;
use crate::layout::point::Point;
use crate::SbmlIssue;

impl SbmlValidable for Point {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<SId>, meta_ids: &mut HashSet<MetaId>) {
        validate_sbase(self, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for Point{}