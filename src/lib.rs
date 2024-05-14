use std::collections::HashSet;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use xml_doc::{Document, Element, ReadOptions};

use xml::{OptionalChild, RequiredProperty};

use crate::constants::namespaces::URL_SBML_CORE;
use crate::core::validation::type_check::{internal_type_check, CanTypeCheck};
use crate::core::validation::{
    apply_rule_10301, apply_rule_10307, apply_rule_10308, apply_rule_10309, apply_rule_10310,
    apply_rule_10312, SbmlValidable,
};
use crate::core::{Model, SBase};
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlDocument, XmlElement, XmlWrapper};

/// A module with useful types that are not directly part of the SBML specification, but help
/// us work with XML documents in a sane and safe way. In particular:
///  - [XmlDocument] | A thread and memory safe reference to a [Document].
///  - [XmlElement] | A thread and memory safe reference to an [Element].
///  - [XmlWrapper] | A trait with utility functions for working with types
///  derived from [XmlElement].
///  - [xml::XmlDefault] | An extension of [XmlWrapper] which allows creation of "default"
///  value for the derived type.
///  - [xml::XmlProperty] and [xml::XmlPropertyType] | Traits providing an abstraction for
///  accessing properties stored in XML attributes. Implementation can be generated using a derive
///  macro.
///  - [xml::XmlChild] and [xml::XmlChildDefault] | Trait abstraction for accessing singleton
///  child tags. Implementation can be generated using a derive macro.
///  - [xml::XmlList] | A generic implementation of [XmlWrapper] which represents
///  a typed list of elements.
///  - [xml::DynamicChild] and [xml::DynamicProperty] | Generic implementations of
///  [xml::XmlProperty] and [xml::XmlChild] that can be used when the name of the property/child
///  is not known at compile time.
pub mod xml;

pub mod core;

pub mod constants;

#[cfg(test)]
pub mod test_suite;

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
    pub fn read_str(file_contents: &str) -> Result<Sbml, String> {
        // Only accept documents that are using UTF-8.
        let opts = ReadOptions {
            enforce_encoding: true,
            encoding: Some("UTF-8".to_string()),
            ..Default::default()
        };
        let doc = match Document::parse_str_with_opts(file_contents, opts) {
            Ok(doc) => doc,
            Err(why) => {
                return if matches!(why, xml_doc::Error::CannotDecode) {
                    Err("SBML documents must use UTF-8 encoding.".to_string())
                } else {
                    Err(why.to_string())
                }
            }
        };
        let root = doc.root_element().unwrap();
        let xml_document = Arc::new(RwLock::new(doc));
        Ok(Sbml {
            xml: xml_document.clone(),
            sbml_root: XmlElement::new_raw(xml_document, root),
        })
    }

    pub fn read_path(path: &str) -> Result<Sbml, String> {
        let file_contents = match std::fs::read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(why) => return Err(why.to_string()),
        };
        Self::read_str(file_contents.as_str())
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
        OptionalChild::new(&self.sbml_root, "model", URL_SBML_CORE)
    }

    pub fn level(&self) -> RequiredProperty<u32> {
        RequiredProperty::new(&self.sbml_root, "level")
    }

    pub fn version(&self) -> RequiredProperty<u32> {
        RequiredProperty::new(&self.sbml_root, "version")
    }

    /// Perform a basic type checking procedure. If this procedure passes without issues,
    /// the document is safe to work with. If some issues are found, working with the document
    /// can cause the program to panic.
    ///
    /// Note that [Sbml::validate] internally also performs a type check before running the full
    /// validation. Hence, a document is also safe to work with if [Sbml::validate] completes
    /// with no issues.
    fn type_check(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.xml.read().unwrap();
        let element = self.sbml_root.raw_element();

        // For the root SBMl element, there are a few extra conditions related to the rule 10102.
        if doc.container().child_elements(doc.deref()).len() != 1 {
            let container = XmlElement::new_raw(self.xml.clone(), doc.container());
            let message = "The document contains multiple root nodes. \
                Only one root <sbml> object is allowed.";
            issues.push(SbmlIssue::new_error("10102", &container, message));
        }

        let root_element = self.sbml_root.xml_element();
        if root_element.tag_name() != "sbml" {
            let message = format!("Invalid root element <{}> found.", root_element.tag_name());
            issues.push(SbmlIssue::new_error("10102", &self.sbml_root, message));
        }

        internal_type_check(&self.sbml_root, issues);

        if element.name(doc.deref()) == "sbml"
            && !element.namespace_decls(doc.deref()).contains_key("")
        {
            issues.push(SbmlIssue::new_error(
                "SANITY_CHECK",
                &self.sbml_root,
                "Sanity check failed: missing required namespace declaration [xmlns] on <sbml>.",
            ));
        }

        if let Some(model) = self.model().get() {
            model.type_check(issues);
        }
    }

    /// Validates the document against validation rules specified in the
    /// [specification](https://sbml.org/specifications/sbml-level-3/version-2/core/release-2/sbml-level-3-version-2-release-2-core.pdf).
    /// Eventual issues are returned in the vector. Empty vector represents a valid document.
    /// ### Rule 10101
    /// is already satisfied implicitly by the use of the package *xml-doc* as writing
    /// is done only in UTF-8 and reading produces error if encoding is different from UTF-8,
    /// UTF-16, ISO 8859-1, GBK or EUC-KR. The specific error is currently covered
    /// in [Self::read_str].
    ///
    /// ### Rule 10104
    /// is already satisfied implicitly by the use of the package *xml-doc* as loading
    /// a document without an error ensures that the document conforms to the basic
    /// structural and syntactic constraints.
    pub fn validate(&self) -> Vec<SbmlIssue> {
        let mut issues: Vec<SbmlIssue> = vec![];
        self.type_check(&mut issues);

        if !issues.is_empty() {
            return issues;
        }

        let mut identifiers: HashSet<String> = HashSet::new();
        let mut meta_ids: HashSet<String> = HashSet::new();

        let xml_element = self.xml_element();
        let id = self.id();
        let meta_id = self.meta_id();

        apply_rule_10301(id.get(), xml_element, &mut issues, &mut identifiers);
        apply_rule_10307(meta_id.get(), xml_element, &mut issues, &mut meta_ids);
        apply_rule_10308(self.sbo_term().get(), xml_element, &mut issues);
        apply_rule_10309(meta_id.get(), xml_element, &mut issues);
        apply_rule_10310(id.get(), xml_element, &mut issues);
        apply_rule_10312(self.name().get(), xml_element, &mut issues);

        if let Some(model) = self.model().get() {
            model.validate(&mut issues, &mut identifiers, &mut meta_ids);
        }

        issues
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

impl XmlWrapper for Sbml {
    fn xml_element(&self) -> &XmlElement {
        &self.sbml_root
    }

    unsafe fn unchecked_cast<T: XmlWrapper>(element: T) -> Self {
        Sbml {
            xml: element.document(),
            sbml_root: element.xml_element().clone(),
        }
    }
}

impl From<Sbml> for XmlElement {
    fn from(value: Sbml) -> Self {
        value.sbml_root
    }
}

impl SBase for Sbml {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SbmlIssue {
    /// Refers to the "raw" XML element where the issue occurred.
    pub element: Element,
    pub severity: SbmlIssueSeverity,
    pub rule: String,
    pub message: String,
}

impl SbmlIssue {
    /// A helper method to more easily create an [SbmlIssue] with [SbmlIssueSeverity::Error]
    /// severity.
    pub fn new_error<S: ToString, E: XmlWrapper>(rule: &str, element: &E, message: S) -> SbmlIssue {
        SbmlIssue {
            element: element.raw_element(),
            severity: SbmlIssueSeverity::Error,
            rule: rule.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum SbmlIssueSeverity {
    /// An issue that makes the document impossible to read correctly (e.g. a function is
    /// used but not declared).
    Error,
    /// An issue that suggests a possible error but does not necessarily make the document
    /// invalid (e.g. a variable is declared but never used).
    Warning,
    /// A suggestion that would improve the document but does not represent a significant
    /// issue (e.g. a property is included when it does not have to be, or unknown tags
    /// or attributes are present in the document, e.g. due to the use of unofficial extensions).
    Info,
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use crate::constants::namespaces::{NS_EMPTY, NS_HTML, NS_SBML_CORE, URL_EMPTY, URL_SBML_CORE};
    use crate::core::RuleTypes::Assignment;
    use crate::core::{
        AlgebraicRule, AssignmentRule, BaseUnit, Compartment, Constraint, Delay, Event,
        EventAssignment, FunctionDefinition, InitialAssignment, KineticLaw, LocalParameter, Math,
        Model, ModifierSpeciesReference, Parameter, Priority, RateRule, Reaction, Rule, RuleTypes,
        SBase, SimpleSpeciesReference, Species, SpeciesReference, Trigger, Unit, UnitDefinition,
    };
    use crate::xml::{
        OptionalXmlChild, OptionalXmlProperty, RequiredDynamicChild, RequiredDynamicProperty,
        RequiredXmlChild, RequiredXmlProperty, XmlChild, XmlChildDefault, XmlDefault, XmlElement,
        XmlProperty, XmlSubtype, XmlSupertype, XmlWrapper,
    };
    use crate::Sbml;

    /// Checks `SbmlDocument`'s properties such as `version` and `level`.
    /// Additionally, checks if `Model` retrieval returns correct child.
    #[test]
    pub fn test_document() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();

        let level = doc.level().get();
        let version = doc.version().get();

        assert_eq!(
            level, 3,
            "Wrong level of SBML.\nActual: {}\nExpected: {}",
            level, "3"
        );
        assert_eq!(
            version, 1,
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

        build_function_defs(&model);
        build_unit_defs(&model);
        build_compartments(&model);
        build_species(&model);
        build_parameters(&model);
        build_initial_assignments(&model);
        build_rules(&model);
        build_constraints(&model);
        build_reactions(&model);
        build_events(&model);

        let _ = sbml_doc.write_path("test-inputs/sbml_build_test.sbml");

        // Clean up the test file.
        std::fs::remove_file("test-inputs/sbml_build_test.sbml").unwrap();
    }

    fn build_function_defs(model: &Model) {
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

    fn build_species(model: &Model) {
        let species = model.species();
        species.ensure();

        let species = species.get().unwrap();
        species.id().set(Some(&"SpeciesList-ID".to_string()));
        species.name().set(Some(&"SpeciesList-NAME".to_string()));
        species.push(Species::new(
            model.document(),
            &String::from("species-1"),
            &String::from("compartment-1"),
        ));
        species.push(Species::new(
            model.document(),
            &String::from("species-2"),
            &String::from("compartment-2"),
        ));
        species.push(Species::new(
            model.document(),
            &String::from("species-3"),
            &String::from("compartment-3"),
        ));

        let species_top = species.top();
        species_top.initial_amount().set(Some(&10.0));
        species_top.initial_concentration().set(Some(&0.5));
        species_top
            .substance_units()
            .set(Some(&BaseUnit::Sievert.to_string()));
        species_top.has_only_substance_units().set(&false);
        species_top.boundary_condition().set(&true);
        species_top.constant().set(&false);
        species_top
            .conversion_factor()
            .set(Some(&"linear".to_string()));
    }

    fn build_parameters(model: &Model) {
        let parameters = model.parameters();
        parameters.ensure();

        let parameters = parameters.get().unwrap();
        parameters.id().set(Some(&"ParamsList-ID".to_string()));
        parameters.name().set(Some(&"ParamsList-NAME".to_string()));
        parameters.push(Parameter::new(
            model.document(),
            &String::from("param-1"),
            true,
        ));
        parameters.push(Parameter::new(
            model.document(),
            &String::from("param-2"),
            true,
        ));

        let param_top = parameters.top();
        param_top.value().set(Some(&15.0));
        param_top.units().set(Some(&BaseUnit::Ampere.to_string()));
    }

    fn build_initial_assignments(model: &Model) {
        let assignments = model.initial_assignments();
        assignments.ensure();

        let assignments = assignments.get().unwrap();
        assignments
            .id()
            .set(Some(&"InitialAssignmentsList-ID".to_string()));
        assignments
            .name()
            .set(Some(&"InitialAssignmentsList-NAME".to_string()));
        assignments.push(InitialAssignment::new(model.document(), &String::from("x")));
        assignments.push(InitialAssignment::new(model.document(), &String::from("x")));

        assignments.get(0).math().ensure();
        assignments.get(1).math().ensure();
    }

    fn build_rules(model: &Model) {
        let rules = model.rules();
        rules.ensure();

        let rules = rules.get().unwrap();
        rules.id().set(Some(&"RulesList-ID".to_string()));
        rules.name().set(Some(&"RulesList-NAME".to_string()));
        rules.push(AlgebraicRule::default(model.document()).upcast());
        rules.push(AssignmentRule::new(model.document(), &String::from("z")).upcast());
        rules.push(RateRule::new(model.document(), &String::from("r")).upcast());

        let algebraic: AlgebraicRule = rules.get(0).downcast();
        algebraic.id().set(Some(&"rule-1".to_string()));
        algebraic.name().set(Some(&"algebraic".to_string()));

        let assignment: AssignmentRule = rules.get(1).downcast();
        assignment.id().set(Some(&"rule-2".to_string()));
        assignment.name().set(Some(&"assignment".to_string()));

        let rate: RateRule = rules.get(2).downcast();
        rate.id().set(Some(&"rule-3".to_string()));
        rate.name().set(Some(&"rate".to_string()));
    }

    fn build_constraints(model: &Model) {
        let constraints = model.constraints();
        constraints.ensure();

        let constraints = constraints.get().unwrap();
        constraints
            .id()
            .set(Some(&"ConstraintsList-ID".to_string()));
        constraints
            .name()
            .set(Some(&"ConstraintsList-NAME".to_string()));
        constraints.push(Constraint::default(model.document()));
        constraints.push(Constraint::default(model.document()));

        constraints
            .get(0)
            .id()
            .set(Some(&"constraint-1".to_string()));
        constraints
            .get(1)
            .id()
            .set(Some(&"constraint-2".to_string()));

        let constraint_top = constraints.top();
        constraint_top.message().set(XmlElement::new_quantified(
            model.document(),
            "message",
            NS_HTML,
        ));
        constraint_top.math().ensure();
    }

    fn build_reactions(model: &Model) {
        let reactions = model.reactions();
        reactions.ensure();

        let reactions = reactions.get().unwrap();
        reactions.id().set(Some(&"ReactionsList-ID".to_string()));
        reactions
            .name()
            .set(Some(&"ReactionsList-NAME".to_string()));
        reactions.push(Reaction::new(
            model.document(),
            &String::from("reaction-1"),
            true,
        ));

        let reaction = reactions.top();
        reaction
            .compartment()
            .set(Some(&"compartment-1".to_string()));

        let reactants = reaction.reactants();
        reactants.ensure();
        let reactants = reactants.get().unwrap();
        reactants.id().set(Some(&"ReactantsList-ID".to_string()));
        reactants.push(SpeciesReference::new(
            model.document(),
            &String::from("species-1"),
            true,
        ));
        let reactant = reactants.top();
        reactant.stoichiometry().set(Some(&2.0));

        let products = reaction.products();
        products.ensure();
        let products = products.get().unwrap();
        products.id().set(Some(&"ProductsList-ID".to_string()));
        products.push(SpeciesReference::new(
            model.document(),
            &String::from("species-1"),
            true,
        ));
        let product = products.top();
        product.stoichiometry().set(Some(&1.0));

        let modifiers = reaction.modifiers();
        modifiers.ensure();
        let modifiers = modifiers.get().unwrap();
        modifiers.id().set(Some(&"ModifiersList-ID".to_string()));
        modifiers.push(ModifierSpeciesReference::new(
            model.document(),
            &String::from("species-2"),
        ));

        let kinetic_law = reaction.kinetic_law();
        kinetic_law.set(KineticLaw::default(model.document()));
        kinetic_law.get().unwrap().math().ensure();
        let kinetic_law = kinetic_law.get().unwrap();
        let local_params = kinetic_law.local_parameters();
        local_params.ensure();
        let local_params = local_params.get().unwrap();
        local_params.push(LocalParameter::new(
            model.document(),
            &String::from("localParam-ID"),
        ));
        let param = local_params.top();
        param.value().set(Some(&42.0));
        param.units().set(Some(&"meter".to_string()));
    }

    fn build_events(model: &Model) {
        let events = model.events();
        events.ensure();

        let events = events.get().unwrap();
        events.id().set(Some(&"EventsList-ID".to_string()));
        events.push(Event::default(model.document()));
        events.push(Event::default(model.document()));

        events.get(0).use_values_from_trigger_time().set(&true);
        events.get(1).use_values_from_trigger_time().set(&false);

        let event = events.top();

        event.trigger().set(Trigger::default(model.document()));
        let trigger = event.trigger().get().unwrap();
        trigger.initial_value().set(&true);
        trigger.persistent().set(&true);
        trigger.math().ensure();

        event.priority().set(Priority::default(model.document()));
        let priority = event.priority().get().unwrap();
        priority.math().ensure();

        event.delay().set(Delay::default(model.document()));
        let delay = event.delay().get().unwrap();
        delay.math().ensure();

        let event_assignments = event.event_assignments();
        event_assignments.ensure();
        let event_assignments = event_assignments.get().unwrap();
        event_assignments
            .id()
            .set(Some(&"EventAssignmentsList-ID".to_string()));
        event_assignments.push(EventAssignment::new(model.document(), &String::from("evt")));
        let assignment = event_assignments.top();
        assignment.math().ensure();
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

        match rules.get(0).cast() {
            RuleTypes::Algebraic(_) => assert!(false),
            Assignment(rule) => {
                assert_eq!(rule.variable().get(), "SUMRecTAINF");
                assert!(rule.math().is_set());
            }
            RuleTypes::Rate(_) => assert!(false),
            RuleTypes::Other(_) => assert!(false),
        };

        match rules.top().cast() {
            RuleTypes::Other(_) => assert!(false),
            RuleTypes::Algebraic(_) => assert!(false),
            Assignment(rule) => {
                assert_eq!(rule.variable().get(), "SUMForFoam");
                assert!(rule.math().is_set());
            }
            RuleTypes::Rate(_) => assert!(false),
        }
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
