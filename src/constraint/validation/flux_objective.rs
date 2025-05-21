use crate::constraint::flux_objective::FluxObjective;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, CanTypeCheck};
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SBase, SId};
use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for FluxObjective {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
    }
}

impl CanTypeCheck for FluxObjective {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if self.sbml_root().model().get().unwrap().strict().get() {
            let coefficient = self.coefficient().get();
            if coefficient.is_nan()
                || coefficient.is_infinite()
                || (coefficient.is_infinite() && coefficient.is_sign_negative())
            {
                let message = "When attribute [strict] on the element of <model> is se to true [coefficient] can not be non-finite".to_string();
                issues.push(SbmlIssue::new_error("fbc-20608", self, message))
            }
        }
    }
}
