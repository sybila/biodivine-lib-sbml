use crate::Sbml;
use std::collections::BTreeSet;

/// ### Rule layout-21503
/// A `LineSegment` must specify the two `Point` elements `start` and `end`. No other elements from
/// the Layout namespace are permitted on a `LineSegment`.
///
/// The document carries a curve segment whose `xsi:type` says `LineSegment` and which also has the
/// two base points of a cubic bezier, so each of them is an element the Layout namespace does not
/// permit there: one issue per element. (The document reports other rules as well, because the
/// fixture is a minimal model: its species reference glyph points at a species reference the model
/// does not define.)
#[test]
fn check_layout_21503() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_21503.xml");
    let issues = doc.unwrap().validate().unwrap();

    let segment_issues = issues
        .iter()
        .filter(|issue| issue.rule() == "layout-21503")
        .count();
    assert_eq!(
        segment_issues,
        2,
        "One issue per element the Layout namespace does not permit: {:?}",
        issues.iter().map(|issue| issue.rule()).collect::<Vec<_>>()
    );
}
