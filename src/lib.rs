use crate::model::SbmlModel;
use crate::xml::{XmlDocument, XmlElement};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
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

/// Declares the [SbmlValidate] trait and should also contain other relevant
/// algorithms/implementations for validation.
pub mod validation;

/// The object that "wraps" an XML document in a SBML-specific API.
///
/// This is mostly just the place where you can specify what SBML version and
/// what SBML extensions are being used. The actual content of the SBML model is
/// then managed through the `SbmlModel` struct.
#[derive(Clone, Debug)]
pub struct SbmlDocument {
    xml: XmlDocument,
}

impl SbmlDocument {
    pub fn read_path(path: &str) -> Result<SbmlDocument, String> {
        let file_contents = match std::fs::read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(why) => return Err(why.to_string()),
        };
        let doc = match Document::from_str(file_contents.as_str()) {
            Ok(doc) => doc,
            Err(why) => return Err(why.to_string()),
        };
        Ok(SbmlDocument {
            xml: Arc::new(RwLock::new(doc)),
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

    // TODO: return OptionalChild<SbmlModel> instead of SbmlModel
    pub fn model(&self) -> SbmlModel {
        // TODO:
        //  This is technically not entirely valid because we should check the namespace
        //  of the model element as well, but it's good enough for a demo. Also, some of this
        //  may need better error handling.

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

        SbmlModel::new(XmlElement::new(self.xml.clone(), model_element))
        // SbmlModel {
        //     // Due to the reference-counting implemented in `Arc`, this does not actually create
        //     // a "deep" copy of the XML document. It just creates a new `Arc` reference to the
        //     // same underlying document object.
        //     xml: XmlElement::new(self.xml.clone(), model_element),
        // }
    }

    pub fn xmlns(&self) -> Result<String, String> {
        let doc = self.xml.read().unwrap();
        match doc
            .root_element()
            .unwrap()
            .namespace_decls(doc.deref())
            .get("")
        {
            Some(xmlns) => Ok(xmlns.to_string()),
            None => Err("Required attribute \"namespace\" xmlns not specified.".to_string()),
        }
    }

    pub fn level(&self) -> Result<u32, String> {
        let doc = self.xml.read().unwrap();
        match doc.root_element().unwrap().attribute(doc.deref(), "level") {
            Some(level) => Ok(level.parse().unwrap()),
            None => Err("Required attribute \"level\" not specified.".to_string()),
        }
    }

    pub fn version(&self) -> Result<u32, String> {
        let doc = self.xml.read().unwrap();
        match doc
            .root_element()
            .unwrap()
            .attribute(doc.deref(), "version")
        {
            Some(level) => Ok(level.parse().unwrap()),
            None => Err("Required attribute \"version\" not specified.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Compartment;
    use crate::xml::{
        OptionalXmlChild, OptionalXmlProperty, RequiredDynamicChild, RequiredDynamicProperty,
        RequiredXmlChild, RequiredXmlProperty, XmlChild, XmlElement, XmlProperty, XmlWrapper,
    };
    use crate::{sbase::SBase, SbmlDocument};
    use std::ops::{Deref, DerefMut};
    use xml_doc::Element;

    /// Checks `SbmlDocument`'s properties such as `xmlns`, `version` and `level`.
    /// Additionaly checks if `Model` retrieval returns correct child.
    #[test]
    pub fn test_document() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();

        let xmlns = doc.xmlns().unwrap();
        let level = doc.level().unwrap();
        let version = doc.version().unwrap();

        assert_eq!(
            xmlns, "http://www.sbml.org/sbml/level3/version1/core",
            "Wrong xmlns of SBML.\nActual: {}\nExpected: {}",
            xmlns, "http://www.sbml.org/sbml/level3/version1/core"
        );
        assert_eq!(
            level, 3,
            "Wrong level of SBML.\nActual: {}\nExpected: {}",
            level, 3
        );
        assert_eq!(
            version, 1,
            "Wrong version of SBML.\nActual: {}\nExpected: {}",
            version, 1
        );

        let model = doc.model();
        assert_eq!(model.id().get().unwrap(), "model_id", "Wrong model.");
    }

    /// Tests read/write operations on `OptionalProperty<>`.
    /// Attempts to remove and create a new custom `OptionalProperty<>`.
    #[test]
    pub fn test_optional_property() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();
        let property = model.id();

        assert!(property.is_set(), "Id is not set but it should be.");
        assert_eq!(property.name(), "id", "Wrong name of the <id> property.");
        assert_eq!(
            property.element().element(),
            model.element(),
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
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();
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
            property.element().element(),
            model.element(),
            "Wrong underlying element of the required property."
        );
        // try to write and read to/from poperty
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
            "Property shouln't be set at this point."
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
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        // get child
        let notes = model.notes();
        assert!(notes.is_set(), "Notes in Model is not set.");
        assert_eq!(notes.name(), "notes", "Wrong name of Notes child.");
        assert_eq!(
            notes.parent().element(),
            model.element(),
            "Wrong parent of Notes child."
        );
        // get child value
        let notes_elem = notes.get();
        assert!(notes_elem.is_some(), "Notes does not contain any element.");
        assert_eq!(
            notes_elem.unwrap().element().name(model.read_doc().deref()),
            "notes",
            "Wrong name of Notes child."
        );
        // clear child
        let notes_elem = notes.set(None);
        assert!(notes_elem.is_some(), "Old notes child is missing");
        assert!(!notes.is_set(), "Notes are still present after clear.");

        let element = Element::new(model.write_doc().deref_mut(), "notes");
        element.set_text_content(model.write_doc().deref_mut(), "Some new text.");
        let xml_element = XmlElement::new(doc.xml, element);
        // set child
        let notes_elem = notes.set(Some(xml_element));
        assert!(notes_elem.is_none(), "Old Notes should be empty.");
        assert!(notes.is_set(), "Notes should be set.");
    }

    /// Tests get/set operations on `RequiredChild<>`.
    /// Attempts to remove and create a new custom `RequiredChild<>`.
    #[test]
    pub fn test_required_child() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        // get child
        let req_child: RequiredDynamicChild<'_, XmlElement> = model.required_child("required");
        assert!(!req_child.is_set());
        assert_eq!(req_child.name(), "required");
        assert_eq!(req_child.parent().element(), model.element());
        let element = Element::new(model.write_doc().deref_mut(), "required");
        let xml_element = XmlElement::new(doc.xml.clone(), element);
        // set child
        req_child.set_raw(xml_element);
        assert!(req_child.is_set());
        element.set_text_content(model.write_doc().deref_mut(), "Some additional content");
        let xml_element = XmlElement::new(doc.xml.clone(), element);
        let old_child = req_child.set(xml_element);
        assert_eq!(old_child.element(), element);
        assert!(req_child.is_set());
        assert_eq!(
            req_child
                .get()
                .element()
                .text_content(model.read_doc().deref()),
            "Some additional content"
        );
        req_child.clear();
        assert!(!req_child.is_set());
    }

    /// Tests get/set operations on special case of children `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`. Checks if addition/removal/get/set methods work correctly
    /// on lists. Attempts to remove and create a new custom `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`.
    #[test]
    pub fn test_lists() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();
        let list = model.compartments();

        assert!(list.is_set());
        assert_eq!(list.name(), "listOfCompartments");
        assert_eq!(list.parent().element(), model.element());
        let content = list.get();
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(!content.is_empty());
        assert_eq!(content.len(), 1);
        let compartment1 = content.get(0);
        assert_eq!(compartment1.constant().get(), true);
        assert_eq!(compartment1.id().get(), "comp1");
        let compartment2: Compartment = XmlElement::new(
            doc.xml,
            Element::new(model.write_doc().deref_mut(), "compartment"),
        )
        .into();
        compartment2.constant().set_raw("false".to_string());
        compartment2.id().set_raw("comp2".to_string());
        content.insert(1, compartment2.clone());
        assert!(content.len() == 2);
        assert_eq!(content.get(0).element(), compartment1.element());
        assert_eq!(content.get(1).element(), compartment2.element());
        content.remove(0);
        assert!(content.len() == 1);
        assert_eq!(content.get(0).element(), compartment2.element());
        content.push(compartment1.clone());
        assert!(content.len() == 2);
        assert_eq!(content.get(0).element(), compartment2.element());
        assert_eq!(content.get(1).element(), compartment1.element());
        content.pop();
        assert!(content.len() == 1);
        assert_eq!(content.get(0).element(), compartment2.element());
    }
}
