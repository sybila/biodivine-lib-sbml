use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, SId};
use crate::xml::OptionalXmlChild;
use crate::{Sbml, SbmlIssue};
use std::collections::HashSet;

#[test]
pub fn property_type() {
    let doc = Sbml::read_path("test-inputs/fbc_tests/example_fbc.xml");
    let objective = doc
        .unwrap()
        .model()
        .get()
        .unwrap()
        .objectives()
        .get()
        .unwrap()
        .get(0);

    let mut issues: Vec<SbmlIssue> = Vec::new();
    let mut ids: HashSet<SId> = HashSet::new();
    let mut mids: HashSet<MetaId> = HashSet::new();

    objective.validate(&mut issues, &mut ids, &mut mids);

    println!("{:?}", issues);
}
