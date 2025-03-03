use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::{apply_rule_10311, apply_rule_10313, SbmlValidable};
use crate::core::{Compartment, MetaId, SId};
use crate::xml::{OptionalXmlProperty, XmlProperty, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Compartment {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let units = self.units();

        apply_rule_10311(units.name(), units.get_raw(), xml_element, issues);
        apply_rule_10313(units.name(), units.get(), xml_element, issues);
    }
}

impl CanTypeCheck for Compartment {}
