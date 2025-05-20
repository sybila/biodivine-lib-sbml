use std::collections::HashSet;
use crate::core::{MetaId, SId};
use crate::core::validation::SbmlValidable;
use crate::core::validation::type_check::CanTypeCheck;
use crate::qual::Transition;
use crate::SbmlIssue;

impl SbmlValidable for Transition {
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<SId>, meta_ids: &mut HashSet<MetaId>) {
        todo!()
    }
}

impl CanTypeCheck for Transition {}