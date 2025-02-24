use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{FunctionDefinition, SId};
use crate::xml::OptionalXmlChild;
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for FunctionDefinition {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<String>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        if let Some(math) = self.math().get() {
            math.validate(issues);
        }
    }
}

impl CanTypeCheck for FunctionDefinition {}
