use crate::cancellation::CancellationCheck;
use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::{apply_rule_10310, validate_sbase};
use crate::core::validation::type_check::{type_check_of_list, type_check_problems, CanTypeCheck};
use crate::core::validation::{matches_xml_id_pattern, validate_list_of_objects, SbmlValidable};
use crate::core::{Compartment, MetaId, Reaction, SBase, SId, Species, SpeciesReference};
use crate::layout::{
    CompartmentGlyph, GeneralGlyph, GraphicalObject, Layout, ReactionGlyph, ReferenceGlyph,
    SpeciesGlyph, SpeciesReferenceGlyph, TextGlyph,
};
use crate::xml::{
    OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty, RequiredXmlChild, XmlChild,
    XmlElement, XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use crate::SbmlResult;
use std::collections::HashSet;

mod bounding_box;
pub(crate) mod curve;
mod dimensions;
mod point;

#[cfg(test)]
mod tests;

impl SbmlValidable for Layout {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        self.dimensions()
            .get()
            .validate(issues, identifiers, meta_ids, check)?;

        if let Some(list_of_compartment_glyphs) = self.compartment_glyphs().get() {
            validate_list_of_objects(
                &list_of_compartment_glyphs,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }

        if let Some(list_of_additional_graph_obj) = self.additional_graph_obj().get() {
            validate_list_of_objects(
                &list_of_additional_graph_obj,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }

        if let Some(list_of_species_glyphs) = self.species_glyphs().get() {
            validate_list_of_objects(
                &list_of_species_glyphs,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }

        if let Some(list_of_reaction_glyphs) = self.reaction_glyphs().get() {
            validate_list_of_objects(
                &list_of_reaction_glyphs,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }

        if let Some(list_of_text_glyphs) = self.text_glyphs().get() {
            validate_list_of_objects(&list_of_text_glyphs, issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for Layout {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.dimensions().get_raw().is_some() {
            self.dimensions().get().type_check(issues, check)?;
        }

        if let Some(list_of_compartment_glyphs) = self.compartment_glyphs().get() {
            type_check_of_list(&list_of_compartment_glyphs, issues, check)?;
        }

        if let Some(list_of_additional_graph_obj) = self.additional_graph_obj().get() {
            type_check_of_list(&list_of_additional_graph_obj, issues, check)?;
        }

        if let Some(list_of_species_glyphs) = self.species_glyphs().get() {
            type_check_of_list(&list_of_species_glyphs, issues, check)?;
        }

        if let Some(list_of_reaction_glyphs) = self.reaction_glyphs().get() {
            type_check_of_list(&list_of_reaction_glyphs, issues, check)?;
        }

        if let Some(list_of_text_glyphs) = self.text_glyphs().get() {
            type_check_of_list(&list_of_text_glyphs, issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for CompartmentGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let compartment = self.compartment();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &compartment, self, issues);

        if let Some(compartment) = compartment.get() {
            let element = self.find_by_sid::<Compartment>(&compartment);

            if element.is_none()
                || element.clone().unwrap().xml_element().tag_name() != "compartment"
            {
                let message =
                    "Attribute [compartment] does not refer to an existing Compartment element!";
                issues.push(SbmlIssue::new("layout-20508", self.xml_element(), message));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
        Ok(())
    }
}

impl CanTypeCheck for CompartmentGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for SpeciesGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let species = self.species();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &species, self, issues);

        if let Some(species) = species.get() {
            let element = self.find_by_sid::<Species>(&species);

            if element.is_none() {
                let message = "Attribute [species] does not refer to an existing Species element!";
                issues.push(SbmlIssue::new("layout-20608", self.xml_element(), message));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
        Ok(())
    }
}

impl CanTypeCheck for SpeciesGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for ReactionGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reaction = self.reaction();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &reaction, self, issues);

        if let Some(reaction) = reaction.get() {
            let element = self.find_by_sid::<Reaction>(&reaction);

            if element.is_none() || element.clone().unwrap().xml_element().tag_name() != "reaction"
            {
                let message =
                    "Attribute [reaction] does not refer to an existing Reaction element!";
                issues.push(SbmlIssue::new("layout-20708", self.xml_element(), message));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }

        validate_list_of_objects(
            &self.species_reference_glyphs().get(),
            issues,
            identifiers,
            meta_ids,
            check,
        )?;
        Ok(())
    }
}

impl CanTypeCheck for ReactionGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues, check)?;
        }

        if self.species_reference_glyphs().get_raw().is_some() {
            type_check_of_list(&self.species_reference_glyphs().get(), issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for SpeciesReferenceGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let species_ref = self.species_reference();
        let species_glyph = self.species_glyph();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &species_ref, self, issues);

        if let Some(species_reference) = species_ref.get() {
            let element = self.find_by_sid::<SpeciesReference>(&species_reference);

            // Rule 21008: the attribute must name a `SpeciesReference` **or** a
            // `ModifierSpeciesReference` of the model - Layout Section 3.10.1 defines it as "an
            // optional attribute of type SIdRef that allows modelers to connect the
            // SpeciesReferenceGlyph with a particular SpeciesReference (or
            // ModifierSpeciesReference) of the containing model". `find_by_sid` returns whichever
            // element carries the id, so the element's own name is what decides, and the message
            // says which of the two cases it is: an id that names nothing and an id that names
            // something of another type are different modelling mistakes.
            let found = element.as_ref().map(|it| it.xml_element().tag_name());
            if !matches!(
                found.as_deref(),
                Some("speciesReference" | "modifierSpeciesReference")
            ) {
                let message = match found {
                    Some(found) => format!(
                        "The value of the attribute [speciesReference] of a <speciesReferenceGlyph> \
                         must be the identifier of an existing <speciesReference> or \
                         <modifierSpeciesReference> object of the enclosing <model>; \
                         '{species_reference}' is the identifier of a <{found}>."
                    ),
                    None => format!(
                        "The value of the attribute [speciesReference] of a <speciesReferenceGlyph> \
                         must be the identifier of an existing <speciesReference> or \
                         <modifierSpeciesReference> object of the enclosing <model>; \
                         '{species_reference}' is the identifier of no element of the document."
                    ),
                };
                issues.push(SbmlIssue::new("layout-21008", self.xml_element(), message));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
        // A missing or unparsable `speciesGlyph` is reported by the type check of this element, so
        // the reference is only resolved when it is readable.
        let element = XmlProperty::get_checked(&species_glyph)
            .ok()
            .flatten()
            .and_then(|species_glyph| self.find_by_sid::<SpeciesGlyph>(&species_glyph));
        if element.is_none() || element.clone().unwrap().xml_element().tag_name() != "speciesGlyph"
        {
            let message =
                "Attribute [speciesGlyph] does not refer to an existing SpeciesGlyph element!";
            issues.push(SbmlIssue::new("layout-21011", self.xml_element(), message));
        }
        Ok(())
    }
}

impl CanTypeCheck for SpeciesReferenceGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for GeneralGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reference = self.reference();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &reference, self, issues);
        apply_rule_20808(reference.get(), self, issues);

        if let (Some(metaid_ref), Some(reference)) = (metaid_ref.get(), reference.get()) {
            apply_rule_20809(self, metaid_ref, reference, issues);
        }

        if let Some(list_of_sub_glyphs) = self.sub_glyphs().get() {
            validate_list_of_objects(&list_of_sub_glyphs, issues, identifiers, meta_ids, check)?;
        }

        if let Some(list_of_reference_glyphs) = self.reference_glyphs().get() {
            validate_list_of_objects(
                &list_of_reference_glyphs,
                issues,
                identifiers,
                meta_ids,
                check,
            )?;
        }
        Ok(())
    }
}

impl CanTypeCheck for GeneralGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues, check)?;
        }

        if let Some(list_of_sub_glyphs) = self.sub_glyphs().get() {
            type_check_of_list(&list_of_sub_glyphs, issues, check)?;
        }

        if let Some(list_of_reference_glyphs) = self.reference_glyphs().get() {
            type_check_of_list(&list_of_reference_glyphs, issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for ReferenceGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reference = self.reference();
        let glyph = self.glyph();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &reference, self, issues);
        apply_rule_20808(reference.get(), self, issues);

        if let (Some(metaid_ref), Some(reference)) = (metaid_ref.get(), reference.get()) {
            apply_rule_20809(self, metaid_ref, reference, issues);
        }

        // `layout:glyph` is required and its value has to be an `SIdRef`, so the reference can only
        // be resolved when the value is a readable identifier: the crate's property types map a
        // value that does not convert to "no value", which is what a typed read of it panics on. A
        // missing attribute is reported by the type check of this element (as `layout-21110`), and a
        // value that is present but is not a valid identifier is the syntax half of the same rule,
        // which this crate does not implement - so the reference rule is only asked about a value it
        // can answer for.
        if let Some(glyph) = XmlProperty::get_checked(&glyph).ok().flatten() {
            if self.find_by_sid::<GraphicalObject>(&glyph).is_none() {
                let message =
                    "Attribute [glyph] does not refer to an existing GraphicalObject element!";
                issues.push(SbmlIssue::new("layout-20508", self.xml_element(), message));
            }
        }
        Ok(())
    }
}

impl CanTypeCheck for ReferenceGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for TextGlyph {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let origin_of_text = self.origin_of_text();
        let graphical_object = self.graphical_object();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids, check)?;
        }

        apply_glyph_rules(&metaid_ref, &origin_of_text, self, issues);
        apply_rule_20808(origin_of_text.get(), self, issues);

        if let Some(graphical_object) = graphical_object.get() {
            let element = self.find_by_sid::<GraphicalObject>(&graphical_object);

            if element.is_none() {
                let message = "Attribute [graphicalObject] does not refer to an existing element!";
                issues.push(SbmlIssue::new("layout-20911", self.xml_element(), message));
            }
        }

        if let (Some(metaid_ref), Some(origin_of_text)) = (metaid_ref.get(), origin_of_text.get()) {
            apply_rule_20809(self, metaid_ref, origin_of_text, issues);
        }
        Ok(())
    }
}

impl CanTypeCheck for TextGlyph {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        issues.extend(type_check_problems(self.xml_element()));
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues, check)?;
        }
        Ok(())
    }
}

impl SbmlValidable for GraphicalObject {
    fn validate_object(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;

        // Exactly one of the variants matches, and each of them validates its own element (they are
        // the same XML element seen through a narrower wrapper), so the walk of an element is
        // handed to its variant and the base-class walk below runs only for the element that has no
        // variant: a bare `<layout:graphicalObject>`. Validating both would report every issue of the
        // element twice, and the second id of an element would be reported as a duplicate.
        if let Some(glyph) = GeneralGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }
        if let Some(glyph) = TextGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }
        if let Some(glyph) = CompartmentGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }
        if let Some(glyph) = SpeciesGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }
        if let Some(glyph) = ReactionGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }
        if let Some(glyph) = ReferenceGlyph::try_cast_from_super(self) {
            return glyph.validate(issues, identifiers, meta_ids, check);
        }

        validate_sbase(self, issues, identifiers, meta_ids);
        apply_rule_layout_20405(self.meta_id_ref().get_raw(), self.xml_element(), issues);

        // The bounding box is required (rule `layout-20407`), but a document that violates the
        // requirement must not make the walk panic: the missing child is reported by the type
        // check, and the guard here keeps the walk working on the object itself.
        if self.bounding_box().get_raw().is_some() {
            self.bounding_box()
                .get()
                .validate(issues, identifiers, meta_ids, check)?;
        }
        Ok(())
    }
}

impl CanTypeCheck for GraphicalObject {
    fn type_check(
        &self,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()> {
        check.tick()?;
        if self.bounding_box().get_raw().is_some() {
            self.bounding_box().get().type_check(issues, check)?;
        }

        // Every typed variant of this element runs the element check itself (they are the same
        // XML element seen through a narrower wrapper), so running it here as well would report
        // the same element twice. The case that has no typed variant is a bare
        // `<layout:graphicalObject>` - the item of `listOfSubGlyphs` and of
        // `listOfAdditionalGraphicalObjects` - and its own rows of the element tables are inert
        // without this call.
        if let Some(glyph) = GeneralGlyph::try_cast_from_super(self) {
            glyph.type_check(issues, check)?;
        } else if let Some(glyph) = TextGlyph::try_cast_from_super(self) {
            glyph.type_check(issues, check)?;
        } else if let Some(glyph) = CompartmentGlyph::try_cast_from_super(self) {
            glyph.type_check(issues, check)?;
        } else if let Some(glyph) = SpeciesGlyph::try_cast_from_super(self) {
            glyph.type_check(issues, check)?;
        } else if let Some(glyph) = ReactionGlyph::try_cast_from_super(self) {
            glyph.type_check(issues, check)?;
        } else {
            issues.extend(type_check_problems(self.xml_element()));
        }
        Ok(())
    }
}

/// ### Rule 20405
/// The attribute *layout:metaidRef* of a GraphicalObject must be of the data type **IDREF**.
///
/// The rule is the reason a `metaidRef` value is checked against the XML `ID` lexical space at all:
/// the two datatypes share it, and the core rule `10309` that would otherwise be used describes the
/// `metaid` attribute of an SBML object, not this reference.
pub(crate) fn apply_rule_layout_20405(
    value: Option<String>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if !matches_xml_id_pattern(&value) {
        let message = format!(
            "The [metaidRef] attribute value ('{0}') does not conform to the syntax of the IDREF data type.",
            value.unwrap()
        );
        issues.push(SbmlIssue::new("layout-20405", xml_element, message))
    }
}

/// ### Rule 20406
/// MetaidRef attribute must refer to an existing component of the model.
pub fn apply_rule_20406<T: SBase>(
    meta_id: Option<MetaId>,
    element: &T,
    issues: &mut Vec<SbmlIssue>,
    xml_element: &XmlElement,
) {
    if meta_id.is_some() {
        match element.find_element_by_meta_id(&meta_id.unwrap()) {
            Some(_) => (),
            None => {
                let message = "Attribute [MetaidRef] does not refer to an existing element!";
                issues.push(SbmlIssue::new("layout-20406", xml_element, message));
            }
        }
    }
}

/// ### Rule 20808
/// Specified glyphs have an attribute SId_reference of corresponding name referencing an existing element.
pub fn apply_rule_20808<T: SBase>(id: Option<SId>, element: &T, issues: &mut Vec<SbmlIssue>) {
    if let Some(id) = id {
        match element.find_element_by_sid(&id) {
            Some(_) => (),
            None => {
                let message =
                    "Attribute containing SId reference does not refer to an existing element!";
                issues.push(SbmlIssue::new(
                    "layout-20808",
                    element.xml_element(),
                    message,
                ));
            }
        }
    }
}

/// ### Rule 20509
/// Both attributes `metaidRef` and attribute holding the metaid reference
/// have to refer to the same specific element of model.
pub fn apply_rule_20509<T: SBase>(
    element: &Option<T>,
    metaid_ref: OptionalSbmlProperty<MetaId>,
    xml_element: &XmlElement,
    issues: &mut Vec<SbmlIssue>,
) {
    if let Some(element) = element {
        let found_mid = element.find_by_meta_id::<T>(&metaid_ref.get().unwrap());

        if found_mid.is_some() && found_mid.unwrap().meta_id().get() == element.meta_id().get() {
            return;
        }
    }
    let message = format!(
        "Attribute [metaidRef] and [{0}] does not refer to the same element!",
        xml_element.full_name()
    );
    issues.push(SbmlIssue::new("layout-20509", xml_element, message))
}

/// ### Rule 20809
/// Same principe as in the rule 20509 but in this case it is not important what element type we
/// are looking for.
pub fn apply_rule_20809<T: SBase>(
    element: &T,
    metaid_ref: MetaId,
    sid_ref: SId,
    issues: &mut Vec<SbmlIssue>,
) {
    let found_mid = element.find_element_by_meta_id(&metaid_ref);
    let found_sid = element.find_element_by_sid(&sid_ref);

    if found_mid.is_some() && found_sid.is_some() && found_mid != found_sid {
        let message = "Attribute [metaidRef] and [compartment] does not refer to the same element!";

        issues.push(SbmlIssue::new(
            "layout-20809",
            element.xml_element(),
            message,
        ))
    }
}

/// Covers the basic rules same for all the `glyph` elements.
///
pub fn apply_glyph_rules<T: SBase>(
    metaid_ref: &OptionalSbmlProperty<MetaId>,
    sid_ref: &OptionalSbmlProperty<SId>,
    element: &T,
    issues: &mut Vec<SbmlIssue>,
) {
    apply_rule_layout_20405(metaid_ref.get_raw(), element.xml_element(), issues);
    apply_rule_20406(metaid_ref.get(), element, issues, element.xml_element());
    apply_rule_10310(sid_ref.get_raw(), element.xml_element(), issues);
}
