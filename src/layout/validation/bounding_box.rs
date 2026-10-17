use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_problems, CanTypeCheck};
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::layout::bounding_box::BoundingBox;
use crate::xml::{RequiredXmlChild, XmlChild, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for BoundingBox {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        self.position()
            .get()
            .validate(issues, identifiers, meta_ids, check)?;
        self.dimensions()
            .get()
            .validate(issues, identifiers, meta_ids, check)?;

        if self.position().get().z().is_set() && !self.dimensions().get().depth().is_set() {
            let message = "If [z] attribute of [position] is not specified the [depth] attribute of [dimensions] must also not be specified!";
            issues.push(SbmlIssue::new("layout-21305", self, message))
        }
        Ok(())
    }
}

impl CanTypeCheck for BoundingBox {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.position().get_raw().is_some() {
            let point = self.position().get();
            point.type_check(issues, check)?;
        }
        if self.dimensions().get_raw().is_some() {
            let dimensions = self.dimensions().get();
            dimensions.type_check(issues, check)?;
        }
        Ok(())
    }
}
