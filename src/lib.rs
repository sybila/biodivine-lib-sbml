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

    pub fn get_model(&self) -> SbmlModel {
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

    pub fn get_xmlns(&self) -> Result<String, String> {
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

    pub fn get_level(&self) -> Result<u32, String> {
        let doc = self.xml.read().unwrap();
        match doc.root_element().unwrap().attribute(doc.deref(), "level") {
            Some(level) => Ok(level.parse().unwrap()),
            None => Err("Required attribute \"level\" not specified.".to_string()),
        }
    }

    pub fn get_version(&self) -> Result<u32, String> {
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
    use crate::xml::{OptionalXmlProperty, XmlChild, XmlChildOptional, XmlElement, XmlWrapper};
    use crate::{sbase::SBase, SbmlDocument};
    use std::ops::Deref;

    #[test]
    pub fn test_model_id() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.get_model();

        // This is a "qualitative" model so there are no function definitions or units.
        assert!(!model.function_definitions().is_set());
        assert!(!model.unit_definitions().is_set());

        assert!(model.notes().is_set());
        {
            let notes = model.notes().get();
            let body = notes.child::<XmlElement>("body").get();
            let p = body.child::<XmlElement>("p").get();
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
        let doc2 = SbmlDocument::read_path("test-inputs/model-modified.sbml").unwrap();
        let model2 = doc2.get_model();
        assert_eq!(model.id().read(), model2.id().read());
        assert_eq!(doc.to_xml_string(), doc2.to_xml_string());
        std::fs::remove_file("test-inputs/model-modified.sbml").unwrap();
    }
}
