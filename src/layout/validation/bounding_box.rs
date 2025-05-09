use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::layout::bounding_box::BoundingBox;
use crate::xml::{RequiredXmlChild, XmlProperty};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for BoundingBox {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        self.position()
            .get()
            .validate(issues, identifiers, meta_ids);
        self.dimensions()
            .get()
            .validate(issues, identifiers, meta_ids);

        if self.position().get().z().is_set() && !self.dimensions().get().depth().is_set() {
            let message = "If [z] attribute of [position] is not specified the [depth] attribute of [dimensions] must also not be specified!";
            issues.push(SbmlIssue::new_error("layout:21305", self, message))
        }
    }
}

impl CanTypeCheck for BoundingBox {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        let point = self.position().get();
        point.type_check(issues);
        let dimensions = self.dimensions().get();
        dimensions.type_check(issues);
    }
}
