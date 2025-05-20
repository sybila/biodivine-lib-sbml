use crate::constants::namespaces::NS_FBC;
use crate::constraint::FbcSpecies;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{internal_type_check, CanTypeCheck};
use crate::core::validation::{apply_rule_10311, apply_rule_10313, SbmlValidable};
use crate::core::{MetaId, SBase, SId, Species};
use crate::xml::{OptionalXmlProperty, XmlProperty, XmlSubtype, XmlWrapper};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Species {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);
        let xml_element = self.xml_element();
        let sbstnc_units = self.substance_units();

        apply_rule_10311(
            sbstnc_units.simple_name(),
            sbstnc_units.get_raw(),
            xml_element,
            issues,
        );
        apply_rule_10313(
            sbstnc_units.simple_name(),
            sbstnc_units.get(),
            xml_element,
            issues,
        );
    }
}

impl CanTypeCheck for Species {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);

        if self.sbml_root().find_sbml_package(NS_FBC) == Ok("fbc".to_string()) {
            if let Some(fbc_species) = FbcSpecies::try_cast_from_super(self) {
                fbc_species.type_check(issues);
            }
        }
    }
}
