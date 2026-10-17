use crate::cancellation::CancellationCheck;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::layout::curve::CubicBezier;
use crate::layout::curve::XsiType;
use crate::layout::curve::{Curve, LineSegment};
use crate::xml::{
    OptionalXmlChild, RequiredXmlChild, RequiredXmlProperty, XmlChild, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Curve {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);
        validate_list_of_objects(
            &self.curve_segments().get(),
            issues,
            identifiers,
            meta_ids,
            check,
        )?;
        Ok(())
    }
}

impl CanTypeCheck for Curve {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if self.curve_segments().get_raw().is_some() {
            type_check_of_list(&self.curve_segments().get(), issues, check)?
        }
        Ok(())
    }
}

impl SbmlValidable for LineSegment {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        self.start()
            .get()
            .validate(issues, identifiers, meta_ids, check)?;
        self.end()
            .get()
            .validate(issues, identifiers, meta_ids, check)?;

        if let Some(segment) = CubicBezier::try_cast_from_super(self) {
            segment.validate(issues, identifiers, meta_ids, check)?;
        } else if self.xsi_type().get() != XsiType::LineSegment {
            let message = "Attribute [xsi:type] has to be of value LineSegment";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        }
        Ok(())
    }
}

impl CanTypeCheck for LineSegment {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        let element = self.xml_element();

        if let Some(segment) = CubicBezier::try_cast_from_super(self) {
            segment.type_check(issues, check)?;
            return Ok(());
        }

        if self.base_point1().is_set() {
            let message =
                "Sanity check failed: unknown child element [basePoint1] on <curveSegment>";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        } else if self.base_point2().is_set() {
            let message =
                "Sanity check failed: unknown child element [basePoint2] on <curveSegment>";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        }

        internal_type_check(element, issues);
        if self.start().get_raw().is_some() {
            self.start().get().type_check(issues, check)?;
        }
        if self.end().get_raw().is_some() {
            self.end().get().type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for CubicBezier {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        if !self.base_point1().is_set() {
            let message =
                "Sanity check failed: missing required child element [basePoint1] on <curveSegment>";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        } else if !self.base_point2().is_set() {
            let message = "Sanity check failed: missing required child element [basePoint2] on <curveSegment>";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        } else {
            self.base_point1()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
            self.base_point2()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        if self.xsi_type().get() != XsiType::CubicBezier {
            let message = "Attribute [xsi:type] has to be of value CubicBezier";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        }
        Ok(())
    }
}

impl CanTypeCheck for CubicBezier {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        internal_type_check(self.xml_element(), issues);

        if !self.base_point1().is_set() || !self.base_point2().is_set() {
            let message =
                "Missing elements of [basePoint1] or [basePoint2] on element [curveSegment]";
            issues.push(SbmlIssue::new("layout-10402", self, message));
        } else {
            self.base_point1()
                .get()
                .unwrap()
                .type_check(issues, check)?;
            self.base_point2()
                .get()
                .unwrap()
                .type_check(issues, check)?;
        }

        if self.start().get_raw().is_some() {
            self.start().get().type_check(issues, check)?;
        }
        if self.end().get_raw().is_some() {
            self.end().get().type_check(issues, check)?;
        }
        Ok(())
    }
}
