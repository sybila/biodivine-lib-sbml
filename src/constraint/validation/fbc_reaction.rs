use crate::cancellation::CancellationCheck;
use crate::constraint::fbc_reaction::FbcReaction;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::type_check::{internal_type_check, CanTypeCheck};
use crate::core::validation::SbmlValidable;
use crate::core::{MetaId, Parameter, SBase, SId, SimpleSpeciesReference, SpeciesReference};
use crate::xml::{
    OptionalXmlChild, OptionalXmlProperty, RequiredXmlProperty, XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

impl SbmlValidable for FbcReaction {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(gene_product_association) = self.gene_product_association().get() {
            gene_product_association.validate(issues, identifiers, meta_ids, check)?;
        }
        if let Some(lower_bound) = self.lower_flux_bound().get() {
            apply_rule_fbc_flux_bound(self, issues, lower_bound, "fbc-20705", "fbc:lowerFluxBound");
        }

        if let Some(upper_bound) = self.upper_flux_bound().get() {
            apply_rule_fbc_flux_bound(self, issues, upper_bound, "fbc-20706", "fbc:upperFluxBound");
        }

        if self.sbml_root().model().get().unwrap().strict().get() {
            let mut initial_assignments_sid: HashSet<SId> = HashSet::new();
            if !self.lower_flux_bound().is_set() || !self.upper_flux_bound().is_set() {
                let message = "When attribute [strict] is se to true [lowerFluxBound] and [upperFluxBound] has to be set".to_string();
                issues.push(SbmlIssue::new("fbc-20707", self, message))
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
                    issues.push(SbmlIssue::new("fbc-20708", self, message))
                }

                if !lower_bound_parameter.value().is_set()
                    || !upper_bound_parameter.value().is_set()
                    || (lower_bound_parameter.value().get().unwrap().is_nan()
                        || upper_bound_parameter.value().get().unwrap().is_nan())
                {
                    let message = "When attribute [strict] is se to true <Parameter> elements referenced by bounds have to be set and can not be NaN".to_string();
                    issues.push(SbmlIssue::new("fbc-20709", self, message))
                } else if lower_bound_parameter.value().get().unwrap().is_infinite()
                    && !lower_bound_parameter
                        .value()
                        .get()
                        .unwrap()
                        .is_sign_negative()
                {
                    // Rule fbc-20711 is about the value `INF`, not about either infinity: a lower
                    // bound of `-INF` is the most permissive bound there is, and the SBML test suite
                    // has a pass document that sets it (`fbc-20711-pass-00-01-sev1-l3v1.xml`). The
                    // sibling rule fbc-20712 is about `-INF` on the upper bound, and the sign test
                    // below is the same one this rule needs.
                    let message = "When attribute [strict] is se to true [lowerFluxBound] value can not be INF".to_string();
                    issues.push(SbmlIssue::new("fbc-20711", self, message))
                } else if upper_bound_parameter.value().get().unwrap().is_infinite()
                    && upper_bound_parameter
                        .value()
                        .get()
                        .unwrap()
                        .is_sign_negative()
                {
                    let message = "When attribute [strict] is se to true [upperFluxBound] value can not be -INF]".to_string();
                    issues.push(SbmlIssue::new("fbc-20712", self, message))
                } else if lower_bound_parameter.value().get().unwrap()
                    > upper_bound_parameter.value().get().unwrap()
                {
                    let message = "When attribute [strict] is se to true [lowerFluxBound] parameter value must be less or equal to [upperFluxBound] parameter value].".to_string();
                    issues.push(SbmlIssue::new("fbc-20713", self, message))
                } else {
                    let model = self.sbml_root().model().get().unwrap();
                    let initial_assignments = model.initial_assignments().get();
                    if let Some(initial_assignments) = initial_assignments {
                        let is_initial = initial_assignments.iter().any(|ia| {
                            let symbol = ia.symbol();
                            initial_assignments_sid.insert(ia.symbol().get());
                            symbol.get() == lower_bound_parameter.id().get()
                                || symbol.get() == upper_bound_parameter.id().get()
                        });

                        if is_initial {
                            let message = "When attribute [strict] is se to true [Parameter] elements referenced by flux bounds can not be referenced by initial assignment.".to_string();
                            issues.push(SbmlIssue::new("fbc-20710", self, message))
                        }
                    }
                }
            }

            let reaction = self.upcast();

            // The three rules about the species references of a reaction are distinct rules of the
            // specification, so they are reported separately:
            //
            //  * fbc-20714: "the `constant` attribute of `SpeciesReference` elements of a `Reaction`
            //    must be set to `true`";
            //  * fbc-20715: "the value of a `SpeciesReference`'s `stoichiometry` attribute must not
            //    be set to `NaN`, `-INF` or `INF`";
            //  * fbc-20716: "the `SpeciesReference` elements of a `Reaction` may not be targeted by
            //    an `InitialAssignment`".
            //
            // They used to share one report (under fbc-20714) and one short-circuiting iteration,
            // which reported the *first* rule and hid the others: the corpus has a document per rule
            // (`fbc-20714-fail-01-01` has a non-constant reference, `fbc-20715-fail-01-0*` have a
            // non-finite stoichiometry) and each expects its own id. The references are walked once
            // and the two list-level conditions are reported at most once per list, which is the
            // granularity the suite's expectation files use.
            for references in [reaction.reactants().get(), reaction.products().get()]
                .into_iter()
                .flatten()
            {
                let mut has_non_constant = false;
                let mut has_non_finite_stoichiometry = false;
                for reference in references.iter() {
                    apply_rule_fbc_20716(&reference, &initial_assignments_sid, issues);
                    has_non_constant |= !reference.constant().get();
                    has_non_finite_stoichiometry |= reference
                        .stoichiometry()
                        .get()
                        .is_some_and(|it| it.is_nan() || it.is_infinite());
                }

                if has_non_constant {
                    let message = "When attribute [strict] is se to true the [constant] attribute \
                                   of <speciesReference> elements have to be set to true."
                        .to_string();
                    issues.push(SbmlIssue::new("fbc-20714", self, message))
                }

                if has_non_finite_stoichiometry {
                    let message = "When attribute [strict] is se to true the [stoichiometry] of \
                                   <speciesReference> elements can not be non-finite."
                        .to_string();
                    issues.push(SbmlIssue::new("fbc-20715", self, message))
                }
            }
        }
        Ok(())
    }
}

impl CanTypeCheck for FbcReaction {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if let Some(gene_product_association) = self.gene_product_association().get() {
            gene_product_association.type_check(issues, check)?;
        } else {
            internal_type_check(self.xml_element(), issues)
        }
        Ok(())
    }
}

/// ### Rules fbc-20705 and fbc-20706
///
/// "The value of the attribute `fbc:lowerFluxBound` (`fbc-20705`) / `fbc:upperFluxBound`
/// (`fbc-20706`) of a `Reaction` object must be the identifier of an existing `Parameter` object
/// defined in the enclosing `Model` object."
///
/// The two rules are the same requirement about two attributes, so they share one check and differ
/// only in the rule id and the attribute the message names - which is what makes the report
/// actionable, and what the SBML test suite compares: `fbc-20705-fail-01-01` has a `lowerFluxBound`
/// that names no parameter and `fbc-20706-fail-01-01` an `upperFluxBound` that does, and each
/// document expects the rule of the attribute it breaks.
///
/// The identifier is resolved through `find_by_sid`, which reports the reference as unresolved when
/// the element it finds is not a `Parameter` at all (the suite has documents where the bound names a
/// species, which is a `Parameter`-shaped reference to the wrong class).
pub fn apply_rule_fbc_flux_bound(
    element: &FbcReaction,
    issues: &mut Vec<SbmlIssue>,
    sid_ref: SId,
    rule: &'static str,
    attribute: &str,
) {
    let found = element.find_by_sid::<Parameter>(&sid_ref);

    if found.is_none() || found.unwrap().xml_element().tag_name() != "parameter" {
        let message = format!(
            "The value of the attribute [{attribute}] of a <reaction> object must be the \
             identifier of an existing <parameter> object defined in the enclosing <model>."
        );
        issues.push(SbmlIssue::new(rule, element.xml_element(), message))
    }
}

fn apply_rule_fbc_20716(
    element: &SpeciesReference,
    initial_sid: &HashSet<SId>,
    issues: &mut Vec<SbmlIssue>,
) {
    if initial_sid.contains(&element.species().get()) {
        let message = "When attribute [strict] is se to true [SpeciesReference] can not be referenced by initial assignment.".to_string();
        issues.push(SbmlIssue::new("fbc-20716", element.xml_element(), message))
    }
}
