use crate::cancellation::CancellationCheck;
use crate::constants::namespaces::{NS_LAYOUT, URL_PACKAGE_LAYOUT};
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{MetaId, SId};
use crate::layout::curve::CubicBezier;
use crate::layout::curve::XsiType;
use crate::layout::curve::{Curve, LineSegment};
use crate::xml::{
    OptionalXmlChild, RequiredXmlChild, RequiredXmlProperty, SbmlProperty, XmlChild, XmlElement,
    XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Curve {
    fn validate_object(
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
        issues.extend(type_check_problems(self.xml_element()));
        // The container is required, so it is checked unconditionally: `RequiredSbmlChild::get`
        // is only reachable when the element is present, and the missing case is reported above.
        if self.curve_segments().get_raw().is_some() {
            type_check_of_list(&self.curve_segments().get(), issues, check)?
        }
        Ok(())
    }
}

impl SbmlValidable for LineSegment {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        // The two points are required (rule `layout-21503`), and a segment without one is reported
        // by the element check of this very element, so the walk only descends into the points it
        // has instead of reading a required child that is not there.
        if self.start().get_raw().is_some() {
            self.start()
                .get()
                .validate(issues, identifiers, meta_ids, check)?;
        }
        if self.end().get_raw().is_some() {
            self.end()
                .get()
                .validate(issues, identifiers, meta_ids, check)?;
        }

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

        // The discriminator decides which variant this element is, and the two variants have
        // different requirements, so it is read first: `layout-10401` when it is missing and
        // `layout-10402` when its value is not one of the two permitted ones. An element whose
        // discriminator is unusable is checked as a line segment, which is also what its element
        // name says.
        let xsi_type = self.xsi_type().get_checked().ok().flatten();

        if xsi_type == Some(XsiType::CubicBezier) {
            if let Some(segment) = CubicBezier::try_cast_from_super(self) {
                segment.type_check(issues, check)?;
            }
            return Ok(());
        }

        check_line_segment_children(self.xml_element(), issues);

        issues.extend(type_check_problems(self.xml_element()));
        if self.start().get_raw().is_some() {
            self.start().get().type_check(issues, check)?;
        }
        if self.end().get_raw().is_some() {
            self.end().get().type_check(issues, check)?;
        }
        Ok(())
    }
}

/// The element-local checks of a curve segment, for the type-check pass of the document *and* for
/// the guard of the object walk: they are the same two rules, and the walk reads the very
/// discriminator they describe, so a use of one without the other would be a hole.
pub(crate) fn check_curve_segment_element(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    if xml_element.tag_name() == "curveSegment"
        && xml_element.namespace_url().as_str() == URL_PACKAGE_LAYOUT
    {
        check_curve_segment_discriminator(xml_element, issues);
    }
}

/// The two rules that describe the `xsi:type` attribute of an abstract curve segment, in one
/// place, because a segment can break at most one of them.
///
/// ### Rule layout-10401
/// The attribute `xsi:type` must be present on all `lineSegment` and `cubicBezier` objects. It is
/// not permitted on any other elements from the SBML Level 3 Layout namespace.
///
/// ### Rule layout-10402
/// The value of the `xsi:type` attribute must be either `LineSegment` or `CubicBezier` appropriate
/// to the object where it is located. No other values are permitted.
///
/// The two halves of `layout-10401` are reported together with `layout-10402` because a caller
/// needs exactly one of three answers: the discriminator, "it is missing" or "its value is not
/// permitted". The "not permitted on any other Layout element" half is decided by the attribute
/// tables of those elements.
pub(crate) fn check_curve_segment_discriminator(
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) -> Option<XsiType> {
    let property = SbmlProperty::<XsiType>::new(xml_element, "xsi:type", NS_LAYOUT, NS_LAYOUT);
    match XmlProperty::get_checked(&property) {
        Ok(Some(xsi_type)) => Some(xsi_type),
        Ok(None) => {
            let message = "The attribute [xsi:type] must be present on a <curveSegment> object.";
            issues.push(SbmlIssue::new("layout-10401", xml_element, message));
            None
        }
        Err(error) => {
            let message = format!(
                "The value of the [xsi:type] attribute is not permitted on a <curveSegment> \
                 object: {error}"
            );
            issues.push(SbmlIssue::new("layout-10402", xml_element, message));
            None
        }
    }
}

/// One way the children of a curve segment can violate the rule of its variant.
///
/// The analysis is shared by the two variants because their rules say the same two things ("must
/// specify these point elements" and "no other elements from the Layout namespace are permitted"),
/// while the rule id, and therefore the report, belongs to the variant: the caller decides which
/// rule id a problem is reported under.
enum CurveSegmentProblem {
    /// A point the variant requires is absent.
    Missing(&'static str),
    /// A point the variant requires appears more than once.
    Repeated(&'static str),
    /// An element from the Layout namespace that the variant does not permit.
    NotPermitted(XmlElement, String),
}

impl CurveSegmentProblem {
    /// The element the problem is reported on and the message that explains it.
    fn describe<'a>(&'a self, segment: &'a XmlElement) -> (&'a XmlElement, String) {
        let element = segment.tag_name();
        match self {
            CurveSegmentProblem::Missing(name) => (
                segment,
                format!("Missing required child <{name}> of the element <{element}>."),
            ),
            CurveSegmentProblem::Repeated(name) => (
                segment,
                format!("Multiple instances of child <{name}> found in element <{element}>."),
            ),
            CurveSegmentProblem::NotPermitted(child, name) => (
                child,
                format!(
                    "Unknown child <{name}> of the element <{element}>: no other element from the \
                     Layout namespace is permitted here."
                ),
            ),
        }
    }
}

/// The children of a curve segment that violate the rule of a variant with the given points.
///
/// The rule names the points the segment must specify ("must specify the two Point elements
/// `start` and `end`", the four of a cubic bezier respectively) and forbids every other element
/// from the Layout namespace, so a missing point, a repeated point and a foreign Layout element are
/// all findings of the same rule. A child in another namespace (the `notes` and `annotation`
/// subobjects of `SBase`) is not a Layout element and is not a finding.
fn curve_segment_problems(
    xml_element: &XmlElement,
    points: &[&'static str],
) -> Vec<CurveSegmentProblem> {
    let children = xml_element.child_elements();
    let mut problems = Vec::new();
    for name in points {
        let mut matching = children
            .iter()
            .filter(|child| child.tag_name().as_str() == *name);
        if matching.next().is_none() {
            problems.push(CurveSegmentProblem::Missing(name));
        } else if matching.next().is_some() {
            problems.push(CurveSegmentProblem::Repeated(name));
        }
    }

    for child in children {
        let name = child.tag_name();
        if child.namespace_url().as_str() == URL_PACKAGE_LAYOUT && !points.contains(&name.as_str())
        {
            problems.push(CurveSegmentProblem::NotPermitted(
                child,
                name.as_str().to_string(),
            ));
        }
    }
    problems
}

/// ### Rule layout-21503
/// A `LineSegment` must specify the two `Point` elements `start` and `end`. No other elements from
/// the Layout namespace are permitted on a `LineSegment`.
fn check_line_segment_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    for problem in curve_segment_problems(xml_element, &["start", "end"]) {
        let (target, message) = problem.describe(xml_element);
        issues.push(SbmlIssue::new("layout-21503", target, message));
    }
}

/// ### Rule layout-21603
/// A `CubicBezier` must specify the four `Point` elements `start`, `basePoint1`, `basePoint2` and
/// `end`. No other elements from the Layout namespace are permitted on a `CubicBezier`.
fn check_cubic_bezier_children(xml_element: &XmlElement, issues: &mut Vec<SbmlIssue>) {
    for problem in
        curve_segment_problems(xml_element, &["start", "basePoint1", "basePoint2", "end"])
    {
        let (target, message) = problem.describe(xml_element);
        issues.push(SbmlIssue::new("layout-21603", target, message));
    }
}

impl SbmlValidable for CubicBezier {
    fn validate_object(
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
        issues.extend(type_check_problems(self.xml_element()));

        check_cubic_bezier_children(self.xml_element(), issues);

        if self.base_point1().is_set() {
            self.base_point1()
                .get()
                .unwrap()
                .type_check(issues, check)?;
        }
        if self.base_point2().is_set() {
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
