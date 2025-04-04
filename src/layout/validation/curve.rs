use std::collections::HashSet;
use crate::core::{MetaId, SId};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::validation::type_check::CanTypeCheck;
use crate::layout::curve::{Curve, LineSegment};
use crate::{SbmlIssue};
use crate::core::validation::sbase::validate_sbase;
use crate::layout::curve::CubicBezier;
use crate::layout::curve::XsiType;
use crate::xml::{RequiredXmlChild, RequiredXmlProperty, XmlSubtype};

impl SbmlValidable for Curve{
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<SId>, meta_ids: &mut HashSet<MetaId>) {
        validate_sbase(self, issues, identifiers, meta_ids);
        validate_list_of_objects(&self.curve_segments().get(), issues, identifiers, meta_ids)
    }
}

impl CanTypeCheck for Curve{}

impl SbmlValidable for LineSegment{
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<SId>, meta_ids: &mut HashSet<MetaId>) {
        validate_sbase(self, issues, identifiers, meta_ids);
        
        self.start().get().validate(issues, identifiers, meta_ids);
        self.end().get().validate(issues, identifiers, meta_ids);
        
        if let Some(segment) = CubicBezier::try_cast_from_super(self) {
            segment.validate(issues, identifiers, meta_ids);
        } else if self.xsi_type().get() != XsiType::LineSegment {
            let message = "Attribute [xsi:type] has to be of value LineSegment";
            issues.push(SbmlIssue::new_error("", self, message));
        }
    }
}

impl CanTypeCheck for LineSegment{}

impl SbmlValidable for CubicBezier{
    fn validate(&self, issues: &mut Vec<SbmlIssue>, identifiers: &mut HashSet<SId>, meta_ids: &mut HashSet<MetaId>) {
        validate_sbase(self, issues, identifiers, meta_ids);
        
        self.base_point1().get().validate(issues, identifiers, meta_ids);
        self.base_point2().get().validate(issues, identifiers, meta_ids);
        
        if self.xsi_type().get() != XsiType::CubicBezier {
            let message = "Attribute [xsi:type] has to be of value CubicBezier";
            issues.push(SbmlIssue::new_error("", self, message));
        }
    }
}

impl CanTypeCheck for CubicBezier{}