use crate::model::SbmlModel;
use crate::xml::{XmlDocument, XmlElement};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use xml::{OptionalChild, RequiredProperty};
use xml_doc::Document;

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

pub const URL_SBML_CORE: &str = "http://www.sbml.org/sbml/level3/version1/core";
pub const NS_SBML_CORE: (&str, &str) = ("", URL_SBML_CORE);
pub const URL_HTML: &str = "http://www.w3.org/1999/xhtml";
pub const NS_HTML: (&str, &str) = ("", URL_HTML);
pub const URL_MATHML: &str = "http://www.w3.org/1998/Math/MathML";
pub const NS_MATHML: (&str, &str) = ("", URL_MATHML);
pub const URL_DEFAULT: &str = "";
pub const NS_DEFAULT: (&str, &str) = ("", URL_DEFAULT);

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
    /// Creates a new blank SBML document with initial skeleton consisting of valid
    /// xml header and sbml root element.
    pub fn new() -> Sbml {
        print!("{}", constants::document::SBML_DEFAULT_DOCUMENT);
        let doc = Document::from_str(constants::document::SBML_DEFAULT_DOCUMENT).unwrap();
        let root = doc.root_element().unwrap();
        let xml_document = Arc::new(RwLock::new(doc));
        Sbml {
            xml: xml_document.clone(),
            sbml_root: XmlElement::new_raw(xml_document, root),
        }
    }

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

    pub fn model(&self) -> OptionalChild<SbmlModel> {
        // TODO:
        //  This is technically not entirely valid because we should check the namespace
        //  of the model element as well, but it's good enough for a demo. Also, some of this
        //  may need better error handling.

        /*
        let model_element = {
            // Lock the XML document for reading. The fact that we are doing this in
            // an extra scope is not necessary for correctness, but it makes it easier
            // for the compiler to infer when the lock should be released, hence we
            // won't accidentally hold it longer than necessary (although, this method is
            // so simple it does not really matter).
            let xml = self.xml.read().unwrap();
            // The `xml` variable here is actually a "read guard" object created by the RwLock.
            // However, we should be able to use it more-or-less like any other reference to a
            // `xml_doc::Document` (e.g., we can call `xml.root_element()` like we would on a
            // "raw" `Document` object). The main difference is if we actually need to send it
            // to a function that accepts a "true" &Document reference. In such case, we need to
            // fake it a bit by calling the `.deref` function.
            xml.root_element()
                .unwrap()
                .find(xml.deref(), "model")
                .unwrap()
        };
        */

        OptionalChild::new(&self.sbml_root, "model", URL_SBML_CORE)

        /*
        SbmlModel::new(XmlElement::new(self.xml.clone(), model_element))
        SbmlModel {
            // Due to the reference-counting implemented in `Arc`, this does not actually create
            // a "deep" copy of the XML document. It just creates a new `Arc` reference to the
            // same underlying document object.
            xml: XmlElement::new(self.xml.clone(), model_element),
        }
        */
    }

    pub fn level(&self) -> RequiredProperty<String> {
        RequiredProperty::new(&self.sbml_root, "level")
    }

    pub fn version(&self) -> RequiredProperty<String> {
        RequiredProperty::new(&self.sbml_root, "version")
    }
}

impl Default for Sbml {
    fn default() -> Self {
        Sbml::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Compartment, SbmlModel};
    use crate::xml::{
        OptionalXmlChild, OptionalXmlProperty, RequiredDynamicChild, RequiredDynamicProperty,
        RequiredXmlChild, RequiredXmlProperty, XmlChild, XmlDefault, XmlElement, XmlProperty,
        XmlWrapper,
    };
    use crate::{sbase::SBase, Sbml, NS_DEFAULT, NS_SBML_CORE, URL_DEFAULT};
    use std::ops::{Deref, DerefMut};

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
            notes_elem.unwrap().name(),
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
            model.required_child("required", URL_DEFAULT);
        assert!(req_child.get_raw().is_none());
        assert_eq!(req_child.name(), "required");
        assert_eq!(req_child.parent().raw_element(), model.raw_element());
        let xml_element = XmlElement::new_quantified(model.document(), "required", NS_DEFAULT);
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
        let sbml_doc = Sbml::new();
        let model = sbml_doc.model();
        let new_model = SbmlModel::default(sbml_doc.xml.clone());
        new_model.raw_element().set_text_content(
            new_model.write_doc().deref_mut(),
            "This is a SBML model element",
        );
        model.set(new_model);
        let model_raw = model.get().unwrap();
        model_raw.id().set(Some(&"model_id".to_string()));

        let _ = sbml_doc.write_path("test-inputs/sbml_build_test.sbml");
        // Clean up the test file.
        std::fs::remove_file("test-inputs/sbml_build_test.sbml").unwrap();
    }
}
