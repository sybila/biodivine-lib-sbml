use crate::core::validation::type_check::CanTypeCheck;
use crate::core::validation::{
    apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309, apply_rule_10310,
    apply_rule_10311, apply_rule_10312, apply_rule_10313, apply_rule_10401, apply_rule_10402,
    SbmlValidable,
};
use crate::core::{Model, Reaction, SBase, SimpleSpeciesReference, Species, SpeciesReference};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlProperty, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for Species {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<String>,
        meta_ids: &mut HashSet<String>,
    ) {
        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();
        let sbstnc_units = self.substance_units();

        apply_rule_10301(Some(id.get()), xml_element, issues, identifiers);
        apply_rule_10307(meta_id.get(), xml_element, issues, meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, issues);
        apply_rule_10309(meta_id.get(), xml_element, issues);
        apply_rule_10310(Some(id.get()), xml_element, issues);
        apply_rule_10311(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);
        apply_rule_10312(self.name().get(), xml_element, issues);
        apply_rule_10313(sbstnc_units.name(), sbstnc_units.get(), xml_element, issues);

        if let Some(annotation) = self.annotation().get() {
            apply_rule_10401(&annotation, issues);
            apply_rule_10402(&annotation, issues);
        }
    }
}

impl CanTypeCheck for Species {}

impl Species {
    /// Determines if this particular species is referenced as a reactant or product in one or more
    /// [Reaction] objects containing [KineticLaw] objects.
    pub(crate) fn is_referenced_by_reaction(&self, model: &Model) -> bool {
        let Some(reactions) = model.reactions().get() else {
            return false;
        };
        let reactions = reactions
            .iter()
            .filter(|r| {
                r.kinetic_law().is_set() && (r.products().is_set() || r.reactants().is_set())
            })
            .collect::<Vec<Reaction>>();

        for reaction in reactions {
            let mut species_references: Vec<SpeciesReference> = Vec::new();
            if let Some(reactants) = reaction.reactants().get() {
                species_references.append(&mut reactants.as_vec());
            }
            if let Some(products) = reaction.products().get() {
                species_references.append(&mut products.as_vec());
            }

            for s_reference in species_references {
                if s_reference.species().get() == self.id().get() {
                    return true;
                }
            }
        }
        false
    }
}
