use crate::cancellation::CancellationCheck;
use crate::constants::namespaces::NS_FBC;
use crate::constraint::FbcSpecies;
use crate::core::validation::sbase::validate_sbase;
use crate::core::validation::type_check::{type_check_problems, CanTypeCheck};
use crate::core::validation::{apply_rule_10311, apply_rule_10313, SbmlValidable};
use crate::core::{MetaId, SBase, SId, Species};
use crate::xml::{OptionalXmlProperty, XmlProperty, XmlSubtype, XmlWrapper};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for Species {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
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
        Ok(())
    }
}

impl CanTypeCheck for Species {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));

        if self.sbml_root().declared_package_prefix(NS_FBC) == Ok("fbc".to_string()) {
            if let Some(fbc_species) = FbcSpecies::try_cast_from_super(self) {
                fbc_species.type_check(issues, check)?;
            }
        }
        Ok(())
    }
}
