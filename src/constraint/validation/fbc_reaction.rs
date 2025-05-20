use crate::constraint::fbc_reaction::FbcReaction;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::type_check::{internal_type_check, CanTypeCheck};
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, Parameter, SBase, SId, SimpleSpeciesReference, SpeciesReference};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::HashSet;

impl SbmlValidable for FbcReaction {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        if let Some(gene_product_association) = self.gene_product_association().get() {
            gene_product_association.validate(issues, identifiers, meta_ids);
        }
        if let Some(lower_bound) = self.lower_flux_bound().get() {
            apply_rule_fbc_20705(self, issues, lower_bound);
        }

        if let Some(upper_bound) = self.upper_flux_bound().get() {
            apply_rule_fbc_20705(self, issues, upper_bound);
        }

        if self.sbml_root().model().get().unwrap().strict().get() {
            let mut initial_assignments_sid: HashSet<SId> = HashSet::new();
            if !self.lower_flux_bound().is_set() || !self.upper_flux_bound().is_set() {
                let message = "When attribute [strict] is se to true [lowerFluxBound] and [upperFluxBound] has to be set".to_string();
                issues.push(SbmlIssue::new_error("fbc-20707", self, message))
            } else {
                let lower_bound_parameter = self
                    .find_by_sid::<Parameter>(&self.lower_flux_bound().get().unwrap())
                    .unwrap();
                let upper_bound_parameter = self
                    .find_by_sid::<Parameter>(&self.upper_flux_bound().get().unwrap())
                    .unwrap();

                if !lower_bound_parameter.constant().get()
                    || !upper_bound_parameter.constant().get()
                {
                    let message = "When attribute [strict] is se to true bounds parameters have to be constant".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20708", self, message))
                }

                if !lower_bound_parameter.value().is_set()
                    || !upper_bound_parameter.value().is_set()
                    || (lower_bound_parameter.value().get().unwrap().is_nan()
                        || upper_bound_parameter.value().get().unwrap().is_nan())
                {
                    let message = "When attribute [strict] is se to true <Parameter> elements referenced by bounds have to be set and can not be NaN".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20709", self, message))
                } else if lower_bound_parameter.value().get().unwrap().is_infinite() {
                    let message = "When attribute [strict] is se to true [lowerFluxBound] value can not be INF".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20711", self, message))
                } else if upper_bound_parameter.value().get().unwrap().is_infinite()
                    && upper_bound_parameter
                        .value()
                        .get()
                        .unwrap()
                        .is_sign_negative()
                {
                    let message = "When attribute [strict] is se to true [upperFluxBound] value can not be -INF]".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20712", self, message))
                } else if lower_bound_parameter.value().get().unwrap()
                    > upper_bound_parameter.value().get().unwrap()
                {
                    let message = "When attribute [strict] is se to true [lowerFluxBound] parameter value must be less or equal to [upperFluxBound] parameter value].".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20713", self, message))
                } else {
                    if let Some(initial_assignments) = self
                        .sbml_root()
                        .model()
                        .get()
                        .unwrap()
                        .initial_assignments()
                        .get()
                    {
                        let is_initial = initial_assignments.iter().any(|ia| {
                            let symbol = ia.symbol();
                            initial_assignments_sid.insert(ia.symbol().get());
                            symbol.get() == lower_bound_parameter.id().get()
                                || symbol.get() == upper_bound_parameter.id().get()
                        });

                        if is_initial {
                            let message = "When attribute [strict] is se to true [Parameter] elements referenced by flux bounds can not be referenced by initial assignment.".to_string();
                            issues.push(SbmlIssue::new_error("fbc-20710", self, message))
                        }
                    }
                }
            }

            let reaction = self.upcast();

            if let Some(reactants) = reaction.reactants().get() {
                let is_invalid = reactants.iter().any(|reactant| {
                    apply_rule_fbc_20716(&reactant, &initial_assignments_sid, issues);
                    let stoichiometry = reactant.stoichiometry().get();

                    !reactant.constant().get()
                        || stoichiometry.map_or(false, |s| {
                            s.is_nan()
                                || s.is_infinite()
                                || (s.is_sign_negative() && s.is_infinite())
                        })
                });

                if is_invalid {
                    let message = "When attribute [strict] is se to true the [constant] attribute of <speciesReference> elements have to be set to true and [stoichiometry] can not be non-finite.".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20714", self, message))
                }
            }

            if let Some(products) = reaction.products().get() {
                let is_invalid = products.iter().any(|product| {
                    apply_rule_fbc_20716(&product, &initial_assignments_sid, issues);
                    let stoichiometry = product.stoichiometry().get();

                    !product.constant().get()
                        || stoichiometry.map_or(false, |s| {
                            s.is_nan()
                                || s.is_infinite()
                                || (s.is_sign_negative() && s.is_infinite())
                        })
                });

                if is_invalid {
                    let message = "When attribute [strict] is se to true the [constant] attribute of <speciesReference> elements have to be set to true and [stoichiometry] can not be non-finite.".to_string();
                    issues.push(SbmlIssue::new_error("fbc-20714", self, message))
                }
            }
        }
    }
}

impl CanTypeCheck for FbcReaction {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if let Some(gene_product_association) = self.gene_product_association().get() {
            gene_product_association.type_check(issues);
        } else {
            internal_type_check(self.xml_element(), issues)
        }
    }
}

pub fn apply_rule_fbc_20705(element: &FbcReaction, issues: &mut Vec<SbmlIssue>, sid_ref: SId) {
    let found = element.find_by_sid::<Parameter>(&sid_ref);

    if found.is_none() || found.unwrap().tag_name() != "parameter" {
        let message =
            "Attribute of Flux Bound does not refer to the existing Parameter element!".to_string();
        issues.push(SbmlIssue::new_error(
            "fbc-20705",
            element.xml_element(),
            message,
        ))
    }
}

fn apply_rule_fbc_20716(
    element: &SpeciesReference,
    initial_sid: &HashSet<SId>,
    issues: &mut Vec<SbmlIssue>,
) {
    if initial_sid.contains(&element.species().get()) {
        let message = "When attribute [strict] is se to true [SpeciesReference] can not be referenced by initial assignment.".to_string();
        issues.push(SbmlIssue::new_error(
            "fbc-20716",
            element.xml_element(),
            message,
        ))
    }
}
