use std::collections::HashSet;
use crate::{Sbml, SbmlIssue};
use crate::core::{MetaId, SId};
use crate::core::validation::SbmlValidable;
use crate::xml::{OptionalXmlChild, RequiredXmlChild, RequiredXmlProperty};

#[test]
fn check_21204() {
    let doc = Sbml::read_path("test-inputs/test-layout/test_10402.xml");
    let model = doc.unwrap().model().get().unwrap().layouts().get().unwrap().get(0);
    
    let var = model.reaction_glyphs().get().unwrap().get(0).curve().get().unwrap();
    println!("{:?}", var.curve_segments().get().get(0).start().get().x().get());

    let mut issues: Vec<SbmlIssue> = Vec::new();
    let mut ide: HashSet<SId> = HashSet::new();
    let mut mide: HashSet<MetaId> = HashSet::new();

    model.validate(&mut issues, &mut ide, &mut mide);

    println!("{:#?}", issues);

    assert_eq!(issues.len(), 1);
    let issue = issues.into_iter().next().unwrap();
    assert_eq!(issue.rule.as_str(), "layout-21204");
}