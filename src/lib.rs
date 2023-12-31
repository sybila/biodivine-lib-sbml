use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use xml_doc::Document;

use xml::{OptionalChild, RequiredProperty};

use crate::constants::namespaces::URL_SBML_CORE;
use crate::model::Model;
use crate::validation::{apply_rule_10102, SbmlIssue};
use crate::xml::{XmlDocument, XmlElement};

/// A module with useful types that are not directly part of the SBML specification, but help
/// us work with XML documents in a sane and safe way. In particular:
///  - [XmlDocument] | A thread and memory safe reference to a [Document].
///  - [XmlElement] | A thread and memory safe reference to an [xml_doc::Element].
///  - [xml::XmlWrapper] | A trait with utility functions for working with types
///  derived from [XmlElement].
///  - [xml::XmlDefault] | An extension of [xml::XmlWrapper] which allows creation of "default"
///  value for the derived type.
///  - [xml::XmlProperty] and [xml::XmlPropertyType] | Traits providing an abstraction for
///  accessing properties stored in XML attributes. Implementation can be generated using a derive
///  macro.
///  - [xml::XmlChild] and [xml::XmlChildDefault] | Trait abstraction for accessing singleton
///  child tags. Implementation can be generated using a derive macro.
///  - [xml::XmlList] | A generic implementation of [xml::XmlWrapper] which represents
///  a typed list of elements.
///  - [xml::DynamicChild] and [xml::DynamicProperty] | Generic implementations of
///  [xml::XmlProperty] and [xml::XmlChild] that can be used when the name of the property/child
///  is not known at compile time.
pub mod xml;

pub mod sbase;

pub mod model;

pub mod constants;

/// Declares the [SbmlValidate] trait and should also contain other relevant
/// algorithms/implementations for validation.
pub mod validation;

/// The object that "wraps" an XML document in a SBML-specific API.
///
/// This is mostly just the place where you can specify what SBML version and
/// what SBML extensions are being used. The actual content of the SBML model is
/// then managed through the `SbmlModel` struct.
#[derive(Clone, Debug)]
pub struct Sbml {
    xml: XmlDocument,
    sbml_root: XmlElement,
}

impl Sbml {
    pub fn read_path(path: &str) -> Result<Sbml, String> {
        let file_contents = match std::fs::read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(why) => return Err(why.to_string()),
        };
        let doc = match Document::from_str(file_contents.as_str()) {
            Ok(doc) => doc,
            Err(why) => return Err(why.to_string()),
        };
        let root = doc.root_element().unwrap();
        let xml_document = Arc::new(RwLock::new(doc));
        Ok(Sbml {
            xml: xml_document.clone(),
            sbml_root: XmlElement::new_raw(xml_document, root),
        })
    }

    pub fn write_path(&self, path: &str) -> Result<(), String> {
        let doc = match self.xml.read() {
            Ok(doc) => doc,
            Err(why) => return Err(why.to_string()),
        };
        match doc.write_file(path) {
            Ok(()) => Ok(()),
            Err(why) => Err(why.to_string()),
        }
    }

    pub fn to_xml_string(&self) -> Result<String, String> {
        let doc = match self.xml.read() {
            Ok(doc) => doc,
            Err(why) => return Err(why.to_string()),
        };
        match doc.write_str() {
            Ok(str) => Ok(str),
            Err(why) => Err(why.to_string()),
        }
    }

    pub fn model(&self) -> OptionalChild<Model> {
        // TODO:
        //  This is technically not entirely valid because we should check the namespace
        //  of the model element as well, but it's good enough for a demo. Also, some of this
        //  may need better error handling.
        OptionalChild::new(&self.sbml_root, "model", URL_SBML_CORE)
    }

    pub fn level(&self) -> RequiredProperty<String> {
        RequiredProperty::new(&self.sbml_root, "level")
    }

    pub fn version(&self) -> RequiredProperty<String> {
        RequiredProperty::new(&self.sbml_root, "version")
    }

    /// Validates the document against validation rules specified in the [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf)
    ///
    /// ### Rule 10101
    /// is already satisfied implicitly by the use of the package *xml-doc* as writing
    /// is done only in UTF-8 and reading produces error if encoding is different from UTF-8,
    /// UTF-16, ISO 8859-1, GBK or EUC-KR.
    pub fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        apply_rule_10102(self.xml.read().unwrap().deref(), &self, issues);
    }
}

impl Default for Sbml {
    /// Creates a new blank SBML document with initial skeleton consisting of valid
    /// xml header and sbml root element.
    fn default() -> Self {
        let doc = Document::from_str(constants::document::SBML_DEFAULT_DOCUMENT).unwrap();
        let root = doc.root_element().unwrap();
        let xml_document = Arc::new(RwLock::new(doc));
        Sbml {
            xml: xml_document.clone(),
            sbml_root: XmlElement::new_raw(xml_document, root),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use crate::constants::namespaces::{NS_EMPTY, NS_HTML, NS_SBML_CORE, URL_EMPTY, URL_SBML_CORE};
    use crate::model::{
        BaseUnit, Compartment, Constraint, FunctionDefinition, Math, Model, SimpleSpeciesReference,
        Unit, UnitDefinition,
    };
    use crate::xml::{
        OptionalXmlChild, OptionalXmlProperty, RequiredDynamicChild, RequiredDynamicProperty,
        RequiredXmlChild, RequiredXmlProperty, XmlChild, XmlChildDefault, XmlDefault, XmlElement,
        XmlProperty, XmlWrapper,
    };
    use crate::{sbase::SBase, Sbml};

    /// Checks `SbmlDocument`'s properties such as `version` and `level`.
    /// Additionally checks if `Model` retrieval returns correct child.
    #[test]
    pub fn test_document() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();

        let level = doc.level().get();
        let version = doc.version().get();

        assert_eq!(
            level, "3",
            "Wrong level of SBML.\nActual: {}\nExpected: {}",
            level, "3"
        );
        assert_eq!(
            version, "1",
            "Wrong version of SBML.\nActual: {}\nExpected: {}",
            version, "1"
        );

        let model = doc.model().get().unwrap();
        assert_eq!(model.id().get().unwrap(), "model_id", "Wrong model.");
    }

    /// Tests read/write operations on `OptionalProperty<>`.
    /// Attempts to remove and create a new custom `OptionalProperty<>`.
    #[test]
    pub fn test_optional_property() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let property = model.id();

        assert!(property.is_set(), "Id is not set but it should be.");
        assert_eq!(property.name(), "id", "Wrong name of the <id> property.");
        assert_eq!(
            property.element().raw_element(),
            model.raw_element(),
            "Wrong underlying element of the <id> property."
        );
        // try reading the <id> property
        let property_val = property.get();
        assert!(
            property_val.is_some(),
            "The <id> property is not set but it should be."
        );
        assert_eq!(
            property_val,
            Some("model_id".to_string()),
            "Wrong value of the <id> property."
        );
        // try clearing the <id> property
        property.clear();
        assert!(
            !property.is_set(),
            "The <id> property should be unset (cleared)."
        );
        assert!(property.get().is_none());
        let property_val = property.get();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );
        let property_val = property.get_raw();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );

        // try overwriting the <id> property
        property.set(Some(&"optional_model_id".to_string()));
        let property_val = property.get();
        assert_eq!(
            property_val,
            Some("optional_model_id".to_string()),
            "Wrong value of the <id> property."
        );
        property.set_raw("raw_model_id".to_string());
        let property_val = property.get();
        assert_eq!(
            property_val,
            Some("raw_model_id".to_string()),
            "Wrong value of the <id> property."
        );
    }

    /// Tests read/write operations on `RequiredProperty<>`.
    /// Attempts to remove and create a new custom `RequiredProperty<>`.
    #[test]
    pub fn test_required_property() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // create a new required property
        let property: RequiredDynamicProperty<'_, String> =
            model.required_property("required_property");
        assert!(
            !property.is_set(),
            "Required property shouldn't be set at this point."
        );
        assert_eq!(
            property.name(),
            "required_property",
            "Wrong name of the required property."
        );
        assert_eq!(
            property.element().raw_element(),
            model.raw_element(),
            "Wrong underlying element of the required property."
        );
        // try to write and read to/from property
        property.set(&"REQ_12345".to_string());
        let property_val = property.get();
        assert_eq!(
            property_val, "REQ_12345",
            "Wrong value of required property."
        );
        let property_val = property.get_raw();
        assert!(property_val.is_some());
        assert_eq!(
            property_val,
            Some("REQ_12345".to_string()),
            "Wrong value of the required property."
        );
        // try to clear the property
        property.clear();
        assert!(
            !property.is_set(),
            "Property shouldn't be set at this point."
        );
        // and write a new value to the property
        property.set_raw("new_req_value".to_string());
        let property_val = property.get();
        assert_eq!(
            property_val, "new_req_value",
            "Wrong value of the required property."
        );
    }

    /// Tests get/set operations on `OptionalChild<>`.
    /// Attempts to remove and create a new custom `OptionalChild<>`.
    #[test]
    pub fn test_optional_child() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // get child
        let notes = model.notes();
        assert!(notes.is_set(), "Notes in Model is not set.");
        assert_eq!(notes.name(), "notes", "Wrong name of Notes child.");
        assert_eq!(
            notes.parent().raw_element(),
            model.raw_element(),
            "Wrong parent of Notes child."
        );
        // get child value
        let notes_elem = notes.get();
        assert!(notes_elem.is_some(), "Notes does not contain any element.");
        assert_eq!(
            notes_elem.unwrap().tag_name(),
            "notes",
            "Wrong name of Notes child."
        );
        // clear child
        let notes_elem = notes.clear();
        assert!(notes_elem.is_some(), "Old notes child is missing");
        assert!(!notes.is_set(), "Notes are still present after clear.");

        // set child
        let xml_element = XmlElement::new_quantified(model.document(), "notes", NS_SBML_CORE);
        let notes_elem = notes.set(xml_element);
        assert!(notes_elem.is_none(), "Old Notes should be empty.");
        assert!(notes.is_set(), "Notes should be set.");
    }

    /// Tests get/set operations on `RequiredChild<>`.
    /// Attempts to remove and create a new custom `RequiredChild<>`.
    #[test]
    pub fn test_required_child() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();

        // get child
        let req_child: RequiredDynamicChild<'_, XmlElement> =
            model.required_child("required", URL_EMPTY);
        assert!(req_child.get_raw().is_none());
        assert_eq!(req_child.name(), "required");
        assert_eq!(req_child.parent().raw_element(), model.raw_element());
        let xml_element = XmlElement::new_quantified(model.document(), "required", NS_EMPTY);
        let inner_element = xml_element.raw_element();
        // set child
        req_child.set_raw(xml_element);
        assert!(req_child.get_raw().is_some());
        inner_element.set_text_content(model.write_doc().deref_mut(), "Some additional content");
        let xml_element = XmlElement::new_raw(doc.xml.clone(), inner_element);
        let old_child = req_child.set(xml_element);
        assert_eq!(old_child.raw_element(), inner_element);
        assert!(req_child.get_raw().is_some());
        assert_eq!(
            req_child
                .get()
                .raw_element()
                .text_content(model.read_doc().deref()),
            "Some additional content"
        );
        req_child.clear_raw();
        assert!(req_child.get_raw().is_none());
    }

    /// Tests get/set operations on special case of children `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`. Checks if addition/removal/get/set methods work correctly
    /// on lists. Attempts to remove and create a new custom `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`.
    #[test]
    pub fn test_lists() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let list = model.compartments();

        assert!(list.is_set());
        assert_eq!(list.name(), "listOfCompartments");
        assert_eq!(list.parent().raw_element(), model.raw_element());
        let content = list.get();
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(!content.is_empty());
        assert_eq!(content.len(), 1);
        let compartment1 = content.get(0);
        assert_eq!(compartment1.constant().get(), true);
        assert_eq!(compartment1.id().get(), "comp1");
        let compartment2: Compartment = Compartment::default(compartment1.document());
        compartment2.constant().set_raw("false".to_string());
        compartment2.id().set_raw("comp2".to_string());
        content.insert(1, compartment2.clone());
        assert_eq!(content.len(), 2);
        assert_eq!(content.get(0).raw_element(), compartment1.raw_element());
        assert_eq!(content.get(1).raw_element(), compartment2.raw_element());
        content.remove(0);
        assert_eq!(content.len(), 1);
        assert_eq!(content.get(0).raw_element(), compartment2.raw_element());
        content.push(compartment1.clone());
        assert_eq!(content.len(), 2);
        assert_eq!(content.get(0).raw_element(), compartment2.raw_element());
        assert_eq!(content.get(1).raw_element(), compartment1.raw_element());
        content.pop();
        assert_eq!(content.len(), 1);
        assert_eq!(content.get(0).raw_element(), compartment2.raw_element());
    }

    #[test]
    pub fn test_build_doc() {
        let sbml_doc = Sbml::default();
        let new_model = Model::default(sbml_doc.xml.clone());

        // set default model element
        sbml_doc.model().set(new_model);
        let model = sbml_doc.model().get().unwrap();
        model.id().set(Some(&"model_id".to_string()));
        model.name().set(Some(&"test model No. 1".to_string()));
        model.sbo_term().set(Some(&"FE12309531 TEST".to_string()));
        model.meta_id().set(Some(&"MT-TEST-MODEL-NO1".to_string()));
        model.notes().set(XmlElement::new_quantified(
            model.document(),
            "notes",
            NS_SBML_CORE,
        ));
        model.annotation().set(XmlElement::new_quantified(
            model.document(),
            "annotation",
            NS_SBML_CORE,
        ));

        // set default notes for model
        let notes = model.notes().get().unwrap();
        notes.raw_element().set_text_content(
            notes.write_doc().deref_mut(),
            "This is a SBML model element.",
        );

        // set default annotation for model
        let annotation = model.annotation().get().unwrap();
        annotation.raw_element().set_text_content(
            annotation.write_doc().deref_mut(),
            "This is a SBML annotation element.",
        );

        build_funtion_defs(&model);
        build_unit_defs(&model);
        build_compartments(&model);

        let _ = sbml_doc.write_path("test-inputs/sbml_build_test.sbml");

        // Clean up the test file.
        std::fs::remove_file("test-inputs/sbml_build_test.sbml").unwrap();
    }

    fn build_funtion_defs(model: &Model) {
        let function_defs = model.function_definitions();
        function_defs.ensure();

        let function_defs_list = function_defs.get().unwrap();
        function_defs_list
            .id()
            .set(Some(&"FunDefsList-ID".to_string()));
        function_defs_list
            .name()
            .set(Some(&"FunDefsList-NAME".to_string()));
        function_defs_list.push(FunctionDefinition::default(model.document()));
        function_defs_list.push(FunctionDefinition::default(model.document()));
        function_defs_list.push(FunctionDefinition::default(model.document()));

        function_defs_list
            .get(0)
            .id()
            .set(Some(&"function-def-1".to_string()));
        function_defs_list
            .get(1)
            .id()
            .set(Some(&"function-def-2".to_string()));
        let fd_top = function_defs_list.top();
        fd_top.id().set(Some(&"function-def-3".to_string()));
        fd_top.math().set(Math::default(model.document()));
    }

    fn build_unit_defs(model: &Model) {
        let unit_defs = model.unit_definitions();
        unit_defs.ensure();

        let unit_defs_list = unit_defs.get().unwrap();
        unit_defs_list
            .id()
            .set(Some(&"UnitDefsList-ID".to_string()));
        unit_defs_list
            .name()
            .set(Some(&"UnitDefsList-NAME".to_string()));
        unit_defs_list.push(UnitDefinition::default(model.document()));
        unit_defs_list.push(UnitDefinition::default(model.document()));
        unit_defs_list.push(UnitDefinition::default(model.document()));

        unit_defs_list
            .get(0)
            .id()
            .set(Some(&"unit-def-1".to_string()));
        unit_defs_list
            .get(1)
            .id()
            .set(Some(&"unit-def-2".to_string()));
        let ud_top = unit_defs_list.top();
        ud_top.id().set(Some(&"unit-def-3-length".to_string()));
        ud_top.name().set(Some(&"unit-def-3-length".to_string()));

        // set default list of units for unit definition
        ud_top.units().ensure();
        let units_list = ud_top.units().get().unwrap();
        units_list.push(Unit::default(model.document()));

        let unit = units_list.top();
        unit.kind().set(&BaseUnit::Metre);
    }

    fn build_compartments(model: &Model) {
        let compartments = model.compartments();
        compartments.ensure();

        let compartments = compartments.get().unwrap();
        compartments.id().set(Some(&"CompsList-ID".to_string()));
        compartments.name().set(Some(&"CompsList-NAME".to_string()));
        compartments.push(Compartment::default(model.document()));
        compartments.push(Compartment::default(model.document()));
        compartments.push(Compartment::default(model.document()));

        compartments.get(0).id().set(&"compartment-1".to_string());
        compartments.get(0).constant().set(&false);
        compartments.get(1).id().set(&"compartment-2".to_string());
        compartments.get(1).constant().set(&false);

        let comp_top = compartments.top();
        comp_top.id().set(&"compartment-3".to_string());
        comp_top.spatial_dimensions().set(Some(&3.0));
        comp_top.size().set(Some(&1.0));
        comp_top.units().set(Some(&"volume".to_string()));
        comp_top.constant().set(&true);
    }

    #[test]
    pub fn test_sbase() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model: Model = doc.model().get().unwrap();

        let id = model.id();
        assert!(id.is_set());
        assert_eq!(id.name(), "id");
        assert_eq!(id.element().raw_element(), model.raw_element());
        assert_eq!(id.get().unwrap(), "model_id");
        id.set(Some(&"new_model_id".to_string()));
        assert_eq!(id.get().unwrap(), "new_model_id");
        id.clear();
        assert!(!id.is_set());

        let name = model.name();
        assert!(!name.is_set());
        assert_eq!(name.name(), "name");
        assert_eq!(name.element().raw_element(), model.raw_element());
        name.set(Some(&"model_name".to_string()));
        assert_eq!(name.get().unwrap(), "model_name");
        name.clear();
        assert!(!name.is_set());

        let meta_id = model.meta_id();
        assert!(meta_id.is_set());
        assert_eq!(meta_id.name(), "metaid");
        assert_eq!(meta_id.element().raw_element(), model.raw_element());
        assert_eq!(
            meta_id.get().unwrap(),
            "_174907b7-8e1c-47f3-9a50-bb8e4c6ebd0d"
        );
        meta_id.set(Some(&"new_meta_id_12345".to_string()));
        assert_eq!(meta_id.get().unwrap(), "new_meta_id_12345");
        meta_id.clear();
        assert!(!meta_id.is_set());

        let sbo_term = model.sbo_term();
        assert!(!sbo_term.is_set());
        assert_eq!(sbo_term.name(), "sboTerm");
        assert_eq!(name.element().raw_element(), model.raw_element());
        sbo_term.set(Some(&"model_sbo_term".to_string()));
        assert_eq!(sbo_term.get().unwrap(), "model_sbo_term");
        sbo_term.clear();
        assert!(!sbo_term.is_set());

        let notes = model.notes();
        assert!(notes.is_set());
        assert_eq!(notes.parent().raw_element(), model.raw_element());
        assert_eq!(notes.name(), "notes");
        assert_eq!(notes.namespace_url(), URL_SBML_CORE);
        assert!(notes.get().is_some());

        let annotation = model.annotation();
        assert!(annotation.is_set());
        assert_eq!(annotation.parent().raw_element(), model.raw_element());
        assert_eq!(annotation.name(), "annotation");
        assert_eq!(annotation.namespace_url(), URL_SBML_CORE);
        assert!(annotation.get().is_some());
    }

    #[test]
    pub fn test_function_definitions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let f_defs = model.function_definitions();
        assert!(f_defs.is_set());

        let f_defs = f_defs.get().unwrap();
        assert!(!f_defs.is_empty());
        assert_eq!(f_defs.len(), 46);

        let f_definition = f_defs.get(0);
        assert!(f_definition.annotation().is_set());
        assert!(f_definition.math().is_set());
    }

    #[test]
    pub fn test_unit_definitions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let unit_defs = model.unit_definitions();
        assert!(unit_defs.is_set());

        let unit_defs = unit_defs.get().unwrap();
        assert!(!unit_defs.is_empty());
        assert_eq!(unit_defs.len(), 5);

        let unit_def = unit_defs.get(0);
        let units = unit_def.units();
        assert!(units.is_set());

        let units = units.get().unwrap();
        assert_eq!(units.len(), 1);

        let unit = units.get(0);
        assert_eq!(unit.exponent().get(), 1.0);
        assert_eq!(unit.kind().get(), BaseUnit::Metre);
        assert_eq!(unit.kind().get().to_string(), BaseUnit::Metre.to_string());
        assert_eq!(unit.multiplier().get(), 1.0);
        assert_eq!(unit.scale().get(), 0);
    }

    #[test]
    pub fn test_compartments() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let compartments = model.compartments();
        assert!(compartments.is_set());

        let compartments = compartments.get().unwrap();
        assert!(!compartments.is_empty());
        assert_eq!(compartments.len(), 7);

        let compartment = compartments.get(0);
        assert_eq!(compartment.id().get(), "Intake");
        assert!(!compartment.units().is_set());
        assert!(compartment.constant().get());
        assert!(compartment.size().is_set());
        assert_eq!(compartment.size().get().unwrap(), 1.0);
        assert!(compartment.spatial_dimensions().is_set());
        assert_eq!(compartment.spatial_dimensions().get().unwrap(), 3.0);
        assert!(!compartment.units().is_set());
    }

    #[test]
    pub fn test_species() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let species = model.species();
        assert!(species.is_set());

        let species = species.get().unwrap();
        assert!(!species.is_empty());
        assert_eq!(species.len(), 51);

        let specie = species.get(0);
        assert_eq!(specie.id().get(), "species_1");
        assert_eq!(specie.compartment().get(), "Intake");
        assert!(!specie.initial_amount().is_set());
        assert_eq!(specie.initial_concentration().get().unwrap(), 1051.0);
        assert!(!specie.substance_units().is_set());
        assert!(specie
            .has_only_substance_units()
            .get_checked()
            .unwrap()
            .is_none());
        assert!(specie.boundary_condition().get());
        assert!(specie.constant().get());
        assert!(!specie.conversion_factor().is_set());
        assert!(specie.annotation().is_set());

        let specie_empty = species.pop();
        assert_eq!(specie_empty.id().get(), "HDL");
        assert_eq!(specie_empty.compartment().get(), "Endothelium");
        assert!(!specie_empty.initial_amount().is_set());
        assert_eq!(specie_empty.initial_concentration().get().unwrap(), 0.0);
        assert!(!specie_empty.substance_units().is_set());
        assert!(specie_empty
            .has_only_substance_units()
            .get_checked()
            .unwrap()
            .is_none());
        assert!(!specie_empty.boundary_condition().get());
        assert!(!specie_empty.constant().get());
        assert!(!specie_empty.conversion_factor().is_set());
        assert!(!specie_empty.annotation().is_set());
    }

    #[test]
    pub fn test_parameters() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let parameters = model.parameters();
        assert!(parameters.is_set());

        let parameters = parameters.get().unwrap();
        assert!(!parameters.is_empty());
        assert_eq!(parameters.len(), 65);

        let parameter = parameters.get(0);
        assert!(parameter.constant().get());
        assert_eq!(parameter.id().get(), "alfa7");
        assert_eq!(parameter.name().get().unwrap(), "alfa7");
        assert_eq!(parameter.value().get().unwrap(), 2.8067);
        assert!(!parameter.units().is_set());
        assert!(!parameter.annotation().is_set());

        let parameter = parameters.pop();
        assert!(parameter.constant().get());
        assert_eq!(parameter.id().get(), "M");
        assert_eq!(parameter.name().get().unwrap(), "M");
        assert_eq!(parameter.value().get().unwrap(), 0.0155561);
        assert!(!parameter.units().is_set());
        assert!(!parameter.annotation().is_set());
    }

    #[test]
    pub fn test_rules() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let rules = model.rules();
        assert!(rules.is_set());

        let rules = rules.get().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules.len(), 9);

        // let rule = rules.get(0).downcast();
        // let concrete_rule = match rule {
        //     RuleEnum::Algebraic(rule) => rule,
        //     RuleEnum::Assignment(rule) => rule,
        //     RuleEnum::Rate(rule) => rule,
        //     RuleEnum::Other(rule) => rule,
        // };

        // assert_eq!(rule., "SUMRecTAINF");
        // assert!(rule.math().is_set());
    }

    #[test]
    pub fn test_reactions() {
        let doc =
            Sbml::read_path("test-inputs/cholesterol_metabolism_and_atherosclerosis.xml").unwrap();
        let model = doc.model().get().unwrap();

        let reactions = model.reactions();
        assert!(reactions.is_set());

        let reactions = reactions.get().unwrap();
        assert!(!reactions.is_empty());
        assert_eq!(reactions.len(), 52);

        let reaction = reactions.get(0);
        assert_eq!(reaction.id().get(), "reaction_1");
        assert_eq!(reaction.meta_id().get().unwrap(), "COPASI41");
        assert_eq!(reaction.name().get().unwrap(), "Ingestion");
        assert!(!reaction.reversible().get());
        assert!(reaction.annotation().is_set());
        assert!(!reaction.compartment().is_set());

        let reactants = reaction.reactants();
        let products = reaction.products();
        let modifiers = reaction.modifiers();
        assert!(reactants.is_set());
        assert!(products.is_set());
        assert!(modifiers.is_set());

        let reactants = reactants.get().unwrap();
        let products = products.get().unwrap();
        let modifiers = modifiers.get().unwrap();
        assert!(!reactants.is_empty());
        assert_eq!(reactants.len(), 1);
        assert!(!products.is_empty());
        assert_eq!(products.len(), 1);
        assert!(!modifiers.is_empty());
        assert_eq!(modifiers.len(), 1);

        let reactant = reactants.pop();
        let product = products.pop();
        let modifier = modifiers.pop();
        assert_eq!(reactant.species().get(), "species_1");
        assert!(reactant.constant().get_checked().unwrap().is_none()); // doesn't conform to level3/version1/core
        assert_eq!(reactant.stoichiometry().get().unwrap(), 1.0);
        assert_eq!(product.species().get(), "species_2");
        assert!(product.constant().get_checked().unwrap().is_none()); // doesn't conform to level3/version1/core
        assert_eq!(product.stoichiometry().get().unwrap(), 1.0);
        assert_eq!(modifier.species().get(), "species_1");

        let kinetic_law = reaction.kinetic_law();
        assert!(kinetic_law.is_set());

        let kinetic_law = kinetic_law.get().unwrap();
        assert!(kinetic_law.math().is_set());

        let local_params = kinetic_law.local_parameters();
        assert!(local_params.is_set());

        let local_params = local_params.get().unwrap();
        assert!(!local_params.is_empty());
        assert_eq!(local_params.len(), 1);

        let param = local_params.pop();
        assert_eq!(param.id().get(), "k1");
        assert_eq!(param.name().get().unwrap(), "k1");
        assert_eq!(param.value().get().unwrap(), 1.0);
        assert!(!param.units().is_set());
    }

    #[test]
    pub fn test_initial_assignments() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let initial_asgnmnts = model.initial_assignments();
        assert!(initial_asgnmnts.is_set());

        let initial_asgnmnts = initial_asgnmnts.get().unwrap();
        assert!(!initial_asgnmnts.is_empty());
        assert_eq!(initial_asgnmnts.len(), 1);

        let single_asgnmnt = initial_asgnmnts.pop();
        assert_eq!(single_asgnmnt.symbol().get(), "Susceptible");
        assert!(single_asgnmnt.math().is_set());
    }

    #[test]
    pub fn test_events() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let events = model.events();
        assert!(events.is_set());

        let events = events.get().unwrap();
        assert!(!events.is_empty());
        assert_eq!(events.len(), 1);

        let event = events.top();
        assert_eq!(event.id().get().unwrap(), "Lockdown");
        assert_eq!(event.meta_id().get().unwrap(), "COPASI13");
        assert_eq!(event.name().get().unwrap(), "Lockdown");
        assert!(!event.priority().is_set());
        assert!(!event.delay().is_set());
        assert!(!event.use_values_from_trigger_time().get());

        let trigger = event.trigger();
        assert!(trigger.is_set());

        let trigger = trigger.get().unwrap();
        assert!(trigger.initial_value().get());
        assert!(!trigger.persistent().get());
        assert!(trigger.math().is_set());

        let assgnmnts = event.event_assignments();
        assert!(assgnmnts.is_set());

        let assgnmnts = assgnmnts.get().unwrap();
        assert!(!assgnmnts.is_empty());
        assert_eq!(assgnmnts.len(), 1);

        let evt_assgnmnt = assgnmnts.top();
        assert_eq!(evt_assgnmnt.variable().get(), "epsilon");
        assert!(evt_assgnmnt.math().is_set());
    }

    #[test]
    pub fn test_constraints() {
        let doc = Sbml::read_path("test-inputs/Mukandavire2020.xml").unwrap();
        let model = doc.model().get().unwrap();

        let constraints = model.constraints();
        assert!(!constraints.is_set());
        constraints.ensure();
        assert!(constraints.is_set());
        let constraints = constraints.get().unwrap();

        assert!(constraints.is_empty());
        assert_eq!(constraints.len(), 0);
        assert!(!constraints.id().is_set());
        assert!(!constraints.name().is_set());
        assert!(!constraints.sbo_term().is_set());
        assert!(!constraints.annotation().is_set());
        assert!(!constraints.notes().is_set());
        assert!(!constraints.meta_id().is_set());
        assert_eq!(constraints.namespace_url(), URL_SBML_CORE);

        let single = Constraint::default(model.document());
        constraints.push(single.clone());
        assert!(!constraints.is_empty());
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints.top().raw_element(), single.raw_element());

        let single = constraints.top();
        assert!(!single.math().is_set());
        assert!(!single.message().is_set());

        let math_el = Math::default(model.document());
        let msg_el = XmlElement::new_quantified(model.document(), "message", NS_HTML);
        msg_el.raw_element().set_text_content(
            model.clone().write_doc().deref_mut(),
            "Warning: Unsatisfied constraint.",
        );
        single.math().set(math_el);
        single.message().set(msg_el);
        assert!(single.math().is_set());
        assert!(single.message().is_set());
        assert!(single
            .message()
            .get()
            .unwrap()
            .raw_element()
            .text_content(model.clone().read_doc().deref())
            .starts_with("Warning:"));
    }
}
