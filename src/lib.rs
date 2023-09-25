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
            None => {
                Err("Required attribute \"namespace\" xmlns not specified."
                    .to_string())
            }
        }
    }

    pub fn level(&self) -> Result<u32, String> {
        let doc = self.xml.read().unwrap();
        match doc.root_element().unwrap().attribute(doc.deref(), "level") {
            Some(level) => Ok(level.parse().unwrap()),
            None => {
                Err("Required attribute \"level\" not specified.".to_string())
            }
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
            None => {
                Err("Required attribute \"version\" not specified.".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::{
        OptionalChild, OptionalProperty, OptionalXmlChild, OptionalXmlProperty,
        RequiredXmlChild, XmlChild, XmlElement, XmlProperty, XmlWrapper,
    };
    use crate::{sbase::SBase, SbmlDocument};
    use std::ops::{Deref, DerefMut};
    use xml_doc::Element;

    #[test]
    pub fn test_model_id() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        // This is a "qualitative" model so there are no function definitions or units.
        assert!(!model.function_definitions().is_set());
        assert!(!model.unit_definitions().is_set());

        assert!(model.notes().is_set());
        {
            let notes = model.notes().get().unwrap();
            let body = notes.required_child::<XmlElement>("body").get();
            let p = body.required_child::<XmlElement>("p").get();
            let doc = model.read_doc();
            let content = p.element().text_content(doc.deref());
            assert!(content.starts_with("This model"));
        }

        let original_id = Some("model_id".to_string());
        let modified_id = "model_6431".to_string();
        assert_eq!(original_id, model.id().read());
        model.id().write(Some(&modified_id));
        assert_eq!(modified_id, model.id().read().unwrap());
        std::fs::write("test-inputs/model-modified.sbml", "dummy").unwrap();
        doc.write_path("test-inputs/model-modified.sbml").unwrap();
        let doc2 =
            SbmlDocument::read_path("test-inputs/model-modified.sbml").unwrap();
        let model2 = doc2.model();
        assert_eq!(model.id().read(), model2.id().read());
        assert_eq!(doc.to_xml_string(), doc2.to_xml_string());
        std::fs::remove_file("test-inputs/model-modified.sbml").unwrap();
    }

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
        assert_eq!(model.id().read().unwrap(), "model_id", "Wrong model.");
    }

    /// Tests read/write operations on `OptionalProperty<>` and `RequiredProperty<>`.
    /// Attempts to remove and create a new custom `OptionalProperty<>` and `RequiredProperty<>`.
    /// Additionaly checks if all existing `SBase` properties are correctly read and written.
    #[test]
    pub fn test_properties() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        let property = model.id();

        assert!(property.is_set(), "Id is not set but it should be.");
        assert_eq!(property.name(), "id", "Wrong name of the <id> property.");
        assert_eq!(
            property.element().element().name(model.read_doc().deref()),
            "model",
            "Wrong underlying element of the <id> property."
        );

        // try reading the <id> property
        let property_val = property.read();
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
        let property_val = property.read();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );
        let property_val = property.read_raw();
        assert!(
            property_val.is_none(),
            "The <id> property should be unset and therefore shouldn't contain any value."
        );

        // try overwriting the <id> property
        property.write(Some(&"optional_model_id".to_string()));
        let property_val = property.read();
        assert_eq!(
            property_val,
            Some("optional_model_id".to_string()),
            "Wrong value of the <id> property."
        );
        property.write_raw("raw_model_id".to_string());
        let property_val = property.read();
        assert_eq!(
            property_val,
            Some("raw_model_id".to_string()),
            "Wrong value of the <id> property."
        );
    }

    /// Tests get/set operations on `OptionalChild<>` and `RequiredChild<>`.
    /// Attempts to remove and create a new custom `OptionalChild<>` and `RequiredChild<>`.
    #[test]
    pub fn test_children() {}

    /// Tests get/set operations on special case of children `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`. Checks if addition/removal/get/set methods work correctly
    /// on lists. Attempts to remove and create a new custom `OptionalChild<XmlList>` and
    /// `RequiredChild<XmlList>`.
    #[test]
    pub fn test_lists() {}

    #[test]
    pub fn test_sbase_id() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        let id: OptionalProperty<String> = model.id();

        assert_eq!(
            id.element().element(),
            model.element(),
            "Wrong underlying element.\nActual: {}\nExpected: {}",
            id.element().element().name(id.element().read_doc().deref()),
            "model"
        );
        assert!(id.is_set(), "Property [id] is not set.");
        assert_eq!(
            id.name(),
            "id",
            "Wrong name of the property [id].\nActual: {}\nExpected: {}",
            id.name(),
            "id"
        );
        assert_eq!(
            id.read().unwrap(),
            "model_id",
            "Wrong id of the Model.\nActual: {}\nExpected: {}",
            id.read().unwrap(),
            "model_id"
        );
        assert_eq!(
            id.read_checked().unwrap().unwrap(),
            "model_id",
            "Wrong id of the Model.\nActual: {}\nExpected: {}",
            id.read_checked().unwrap().unwrap(),
            "model_id"
        );
        assert_eq!(
            id.read_raw().unwrap(),
            "model_id",
            "Wrong id of the Model.\nActual: {}\nExpected: {}",
            id.read_raw().unwrap(),
            "model_id"
        );

        id.clear();
        assert!(!id.is_set());
        assert!(id.read().is_none());

        id.write(Some(&"model_id_write".to_string()));
        assert_eq!(
            id.read().unwrap(),
            "model_id_write",
            "Wrong id of the Model.\nActual: {}\nExpected: {}",
            id.read().unwrap(),
            "model_id_write"
        );

        id.write_raw("model_id_write_raw".to_string());
        assert_eq!(
            id.read().unwrap(),
            "model_id_write_raw",
            "Wrong id of the Model.\nActual: {}\nExpected: {}",
            id.read().unwrap(),
            "model_id_write_raw"
        );
    }

    #[test]
    pub fn test_sbase_notes() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model();

        let notes: OptionalChild<XmlElement> = model.notes();

        assert!(notes.is_set());
        {
            let body = notes
                .get()
                .unwrap()
                .required_child::<XmlElement>("body")
                .get();
            let p = body.required_child::<XmlElement>("p").get();
            let content = p.element().text_content(model.read_doc().deref());
            assert!(content.starts_with("This model is an adapted version"));
        }
        assert_eq!(
            notes.name(),
            "notes",
            "Wrong name of the child [notes].\nActual: {}\nExpected: {}",
            notes.name(),
            "notes"
        );
        assert_eq!(
            notes.parent().element(),
            model.element(),
            "Wrong parent of the child [notes].\nActual: {}\nExpected: {}",
            notes
                .parent()
                .element()
                .name(notes.parent().read_doc().deref()),
            "model"
        );

        let removed = notes.clear();
        assert!(removed.is_some());
        assert!(!notes.is_set());
        let removed = removed.unwrap();
        assert!(removed.required_child::<XmlElement>("body").is_set());

        let new_notes = XmlElement::new(
            doc.xml.clone(),
            Element::new(model.write_doc().deref_mut(), "notes"),
        );
        let old_notes = notes.set(Some(new_notes));
        assert!(notes.is_set());
        assert!(old_notes.is_none());
        let notes_xml = notes.get().unwrap();
        let body = notes_xml.required_child::<XmlElement>("body");
        let new_body = XmlElement::new(
            doc.xml.clone(),
            Element::new(model.write_doc().deref_mut(), "body"),
        );
        body.set(new_body); // panics. Unable to create required child.
    }
}
