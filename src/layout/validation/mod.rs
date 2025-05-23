use crate::core::sbase::SbmlUtils;
use crate::core::validation::sbase::{apply_rule_10309, apply_rule_10310, validate_sbase};
use crate::core::validation::type_check::{internal_type_check, type_check_of_list, CanTypeCheck};
use crate::core::validation::{validate_list_of_objects, SbmlValidable};
use crate::core::{Compartment, MetaId, Reaction, SBase, SId, Species, SpeciesReference};
use crate::layout::{
    CompartmentGlyph, GeneralGlyph, GraphicalObject, Layout, ReactionGlyph, ReferenceGlyph,
    SpeciesGlyph, SpeciesReferenceGlyph, TextGlyph,
};
use crate::xml::{
    OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty, RequiredXmlChild,
    RequiredXmlProperty, XmlChild, XmlElement, XmlProperty, XmlSubtype, XmlWrapper,
};
use crate::SbmlIssue;
use std::collections::HashSet;

mod bounding_box;
mod curve;
mod dimensions;
mod point;

#[cfg(test)]
mod tests;

impl SbmlValidable for Layout {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        self.dimensions()
            .get()
            .validate(issues, identifiers, meta_ids);

        if let Some(list_of_compartment_glyphs) = self.compartment_glyphs().get() {
            validate_list_of_objects(&list_of_compartment_glyphs, issues, identifiers, meta_ids);
        }

        if let Some(list_of_additional_graph_obj) = self.additional_graph_obj().get() {
            validate_list_of_objects(&list_of_additional_graph_obj, issues, identifiers, meta_ids);
        }

        if let Some(list_of_species_glyphs) = self.species_glyphs().get() {
            validate_list_of_objects(&list_of_species_glyphs, issues, identifiers, meta_ids);
        }

        if let Some(list_of_reaction_glyphs) = self.reaction_glyphs().get() {
            validate_list_of_objects(&list_of_reaction_glyphs, issues, identifiers, meta_ids);
        }

        if let Some(list_of_text_glyphs) = self.text_glyphs().get() {
            validate_list_of_objects(&list_of_text_glyphs, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for Layout {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if self.dimensions().get_raw().is_some() {
            self.dimensions().get().type_check(issues);
        }

        if let Some(list_of_compartment_glyphs) = self.compartment_glyphs().get() {
            type_check_of_list(&list_of_compartment_glyphs, issues);
        }

        if let Some(list_of_additional_graph_obj) = self.additional_graph_obj().get() {
            type_check_of_list(&list_of_additional_graph_obj, issues);
        }

        if let Some(list_of_species_glyphs) = self.species_glyphs().get() {
            type_check_of_list(&list_of_species_glyphs, issues);
        }

        if let Some(list_of_reaction_glyphs) = self.reaction_glyphs().get() {
            type_check_of_list(&list_of_reaction_glyphs, issues);
        }

        if let Some(list_of_text_glyphs) = self.text_glyphs().get() {
            type_check_of_list(&list_of_text_glyphs, issues);
        }
    }
}

impl SbmlValidable for CompartmentGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let compartment = self.compartment();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &compartment, self, issues);

        if self.compartment().is_set() {
            let element = self.find_by_sid::<Compartment>(&compartment.get().unwrap());

            if element.is_none() || element.clone().unwrap().full_name() != "compartment" {
                let message =
                    "Attribute [compartment] does not refer to an existing Compartment element!";
                issues.push(SbmlIssue::new_error(
                    "layout-20508",
                    self.xml_element(),
                    message,
                ));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
    }
}

impl CanTypeCheck for CompartmentGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }
    }
}

impl SbmlValidable for SpeciesGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let species = self.species();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &species, self, issues);

        if self.species().is_set() {
            let element = self.find_by_sid::<Species>(&species.get().unwrap());

            if element.is_none() {
                let message = "Attribute [species] does not refer to an existing Species element!";
                issues.push(SbmlIssue::new_error(
                    "layout-20608",
                    self.xml_element(),
                    message,
                ));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
    }
}

impl CanTypeCheck for SpeciesGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }
    }
}

impl SbmlValidable for ReactionGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reaction = self.reaction();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &reaction, self, issues);

        if self.reaction().is_set() {
            let element = self.find_by_sid::<Reaction>(&reaction.get().unwrap());

            if element.is_none() || element.clone().unwrap().full_name() != "reaction" {
                let message =
                    "Attribute [reaction] does not refer to an existing Reaction element!";
                issues.push(SbmlIssue::new_error(
                    "layout-20708",
                    self.xml_element(),
                    message,
                ));
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
        );
    }
}

impl CanTypeCheck for ReactionGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues);
        }

        if self.species_reference_glyphs().get_raw().is_some() {
            type_check_of_list(&self.species_reference_glyphs().get(), issues);
        }
    }
}

impl SbmlValidable for SpeciesReferenceGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let species_ref = self.species_reference();
        let species_glyph = self.species_glyph();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &species_ref, self, issues);

        if self.species_reference().is_set() {
            let element = self.find_by_sid::<SpeciesReference>(&species_ref.get().unwrap());

            if element.is_none() || element.clone().unwrap().full_name() != "speciesReference" {
                let message = "Attribute [speciesReference] does not refer to an existing SpeciesReference element!";
                issues.push(SbmlIssue::new_error(
                    "layout-21008",
                    self.xml_element(),
                    message,
                ));
            }

            if self.meta_id_ref().is_set() {
                apply_rule_20509(&element, metaid_ref, self.xml_element(), issues);
            }
        }
        let element = self.find_by_sid::<SpeciesGlyph>(&species_glyph.get());
        if element.is_none() || element.clone().unwrap().full_name() != "layout:speciesGlyph" {
            let message =
                "Attribute [speciesGlyph] does not refer to an existing SpeciesGlyph element!";
            issues.push(SbmlIssue::new_error(
                "layout-21011",
                self.xml_element(),
                message,
            ));
        }
    }
}

impl CanTypeCheck for SpeciesReferenceGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues);
        }
    }
}

impl SbmlValidable for GeneralGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reference = self.reference();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &reference, self, issues);
        apply_rule_20808(reference.get(), self, issues);

        if self.meta_id_ref().is_set() && self.reference().is_set() {
            apply_rule_20809(
                self,
                metaid_ref.get().unwrap(),
                reference.get().unwrap(),
                issues,
            );
        }

        if let Some(list_of_sub_glyphs) = self.sub_glyphs().get() {
            validate_list_of_objects(&list_of_sub_glyphs, issues, identifiers, meta_ids);
        }

        if let Some(list_of_reference_glyphs) = self.reference_glyphs().get() {
            validate_list_of_objects(&list_of_reference_glyphs, issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for GeneralGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues);
        }

        if let Some(list_of_sub_glyphs) = self.sub_glyphs().get() {
            type_check_of_list(&list_of_sub_glyphs, issues);
        }

        if let Some(list_of_reference_glyphs) = self.reference_glyphs().get() {
            type_check_of_list(&list_of_reference_glyphs, issues);
        }
    }
}

impl SbmlValidable for ReferenceGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let reference = self.reference();
        let glyph = self.glyph();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        if let Some(curve) = self.curve().get() {
            curve.validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &reference, self, issues);
        apply_rule_20808(reference.get(), self, issues);

        if self.meta_id_ref().is_set() && self.reference().is_set() {
            apply_rule_20809(
                self,
                metaid_ref.get().unwrap(),
                reference.get().unwrap(),
                issues,
            );
        }

        if self.find_by_sid::<GraphicalObject>(&glyph.get()).is_none() {
            let message =
                "Attribute [glyph] does not refer to an existing GraphicalObject element!";
            issues.push(SbmlIssue::new_error(
                "layout-20508",
                self.xml_element(),
                message,
            ));
        }
    }
}

impl CanTypeCheck for ReferenceGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }

        if let Some(curve) = self.curve().get() {
            curve.type_check(issues);
        }
    }
}

impl SbmlValidable for TextGlyph {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        let metaid_ref = self.meta_id_ref();
        let origin_of_text = self.origin_of_text();
        let graphical_object = self.graphical_object();

        if self.bounding_box().is_set() {
            self.bounding_box()
                .get()
                .unwrap()
                .validate(issues, identifiers, meta_ids);
        }

        apply_glyph_rules(&metaid_ref, &origin_of_text, self, issues);
        apply_rule_20808(origin_of_text.get(), self, issues);

        if graphical_object.is_set() {
            let element = self.find_by_sid::<GraphicalObject>(&graphical_object.get().unwrap());

            if element.is_none() {
                let message = "Attribute [graphicalObject] does not refer to an existing element!";
                issues.push(SbmlIssue::new_error(
                    "layout-20911",
                    self.xml_element(),
                    message,
                ));
            }
        }

        if self.meta_id_ref().is_set() && self.origin_of_text().is_set() {
            apply_rule_20809(
                self,
                metaid_ref.get().unwrap(),
                origin_of_text.get().unwrap(),
                issues,
            );
        }
    }
}

impl CanTypeCheck for TextGlyph {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        internal_type_check(self.xml_element(), issues);
        if self.bounding_box().is_set() {
            let bb = self.bounding_box().get().unwrap();
            bb.type_check(issues);
        }
    }
}

impl SbmlValidable for GraphicalObject {
    fn validate(
        &self,
        issues: &mut Vec<SbmlIssue>,
        identifiers: &mut HashSet<SId>,
        meta_ids: &mut HashSet<MetaId>,
    ) {
        validate_sbase(self, issues, identifiers, meta_ids);

        self.bounding_box()
            .get()
            .validate(issues, identifiers, meta_ids);

        apply_rule_10309(self.meta_id_ref().get_raw(), self.xml_element(), issues);

        if let Some(glyph) = GeneralGlyph::try_cast_from_super(self) {
            glyph.validate(issues, identifiers, meta_ids);
        }

        if let Some(glyph) = TextGlyph::try_cast_from_super(self) {
            glyph.validate(issues, identifiers, meta_ids);
        }

        if let Some(glyph) = CompartmentGlyph::try_cast_from_super(self) {
            glyph.validate(issues, identifiers, meta_ids);
        }

        if let Some(glyph) = SpeciesGlyph::try_cast_from_super(self) {
            glyph.validate(issues, identifiers, meta_ids);
        }

        if let Some(glyph) = ReactionGlyph::try_cast_from_super(self) {
            glyph.validate(issues, identifiers, meta_ids);
        }
    }
}

impl CanTypeCheck for GraphicalObject {
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        if self.bounding_box().get_raw().is_some() {
            self.bounding_box().get().type_check(issues);
        }

        if let Some(glyph) = GeneralGlyph::try_cast_from_super(self) {
            glyph.type_check(issues);
        }

        if let Some(glyph) = TextGlyph::try_cast_from_super(self) {
            glyph.type_check(issues);
        }

        if let Some(glyph) = CompartmentGlyph::try_cast_from_super(self) {
            glyph.type_check(issues);
        }

        if let Some(glyph) = SpeciesGlyph::try_cast_from_super(self) {
            glyph.type_check(issues);
        }

        if let Some(glyph) = ReactionGlyph::try_cast_from_super(self) {
            glyph.type_check(issues);
        }
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
                issues.push(SbmlIssue::new_error("layout-20406", xml_element, message));
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
                issues.push(SbmlIssue::new_error(
                    "layout-20808",
                    element.xml_element(),
                    message,
                ));
            }
        }
    }
}

/// ### Rule 20509
/// Both attributes [metaidRef] and attribute holding the metaid reference
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
    issues.push(SbmlIssue::new_error("layout-20509", xml_element, message))
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

        issues.push(SbmlIssue::new_error(
            "layout-20809",
            element.xml_element(),
            message,
        ))
    }
}

/// Covers the basic rules same for all the [glyph] elements.
///
pub fn apply_glyph_rules<T: SBase>(
    metaid_ref: &OptionalSbmlProperty<MetaId>,
    sid_ref: &OptionalSbmlProperty<SId>,
    element: &T,
    issues: &mut Vec<SbmlIssue>,
) {
    apply_rule_10309(metaid_ref.get_raw(), element.xml_element(), issues);
    apply_rule_20406(metaid_ref.get(), element, issues, element.xml_element());
    apply_rule_10310(sid_ref.get_raw(), element.xml_element(), issues);
}
