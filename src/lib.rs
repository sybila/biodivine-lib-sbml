use crate::xml::{XmlDocument, XmlElement, XmlList, XmlWrapper};
use crate::sbase::SBaseDefault;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use xml_doc::Document;

/// A module with useful types that are not directly part of the SBML specification, but help
/// us work with XML documents in a sane and safe way.
pub mod xml;

pub mod sbase;

/// The object that "wraps" an XML document in a SBML-specific API.
///
/// This is mostly just the place where you can specify what SBML version and
/// what SBML extensions are being used. The actual content of the SBML model is
/// then managed through the `SbmlModel` struct.
#[derive(Clone, Debug)]
pub struct SbmlDocument {
    xml: XmlDocument,
}

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug)]
pub struct SbmlModel {
    xml: XmlElement,
}

impl XmlWrapper for SbmlModel {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

/// Adds the default implementation of [SBase] to the [SbmlModel].
impl SBaseDefault for SbmlModel {}

#[derive(Clone, Debug)]
pub struct SbmlFunctionDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlFunctionDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlFunctionDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlFunctionDefinition { xml }
    }
}

#[derive(Clone, Debug)]
pub struct SbmlUnitDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlUnitDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlUnitDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlUnitDefinition { xml }
    }
}

impl SbmlUnitDefinition {
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

pub struct Unit {
    xml: XmlElement,
}

impl XmlWrapper for Unit {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for Unit {
    fn from(xml: XmlElement) -> Self {
        Unit { xml }
    }
}

impl Unit {
    pub fn get_kind(&self) {
        todo!()
    }

    pub fn get_exponent(&self) {
        todo!()
    }

    pub fn get_scale(&self) {
        todo!()
    }

    pub fn get_multiplier(&self) {
        todo!()
    }
}
/// TODO: If I recall correctly, these should also implement SBase, but remove if that's not true.
impl SBaseDefault for SbmlFunctionDefinition {}

impl SbmlModel {
    pub fn get_function_definitions(&self) -> XmlList<SbmlFunctionDefinition> {
        let list_element = {
            let xml = self.read_doc();
            self.element()
                .find(xml.deref(), "listOfFunctionDefinitions")
                .unwrap()
        };
        XmlList::from(self.as_xml().derive(list_element))
    }

    pub fn get_unit_definitions(&self) -> XmlList<SbmlUnitDefinition> {
        let list = self.child_element("listOfUnitDefinitions");
        XmlList::from(self.as_xml().derive(list))
    }
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
            Err(why) => return Err(why.to_string()),
        }
    }

    pub fn to_xml_string(&self) -> Result<String, String> {
        let doc = match self.xml.read() {
            Ok(doc) => doc,
            Err(why) => return Err(why.to_string()),
        };
        match doc.write_str() {
            Ok(str) => Ok(str),
            Err(why) => return Err(why.to_string()),
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

        SbmlModel {
            // Due to the reference-counting implemented in `Arc`, this does not actually create
            // a "deep" copy of the XML document. It just creates a new `Arc` reference to the
            // same underlying document object.
            xml: XmlElement::new(self.xml.clone(), model_element),
        }
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
            None => return Err("Required attribute \"namespace\" xmlns not specified.".to_string()),
        }
    }

    pub fn get_level(&self) -> Result<u32, String> {
        let doc = self.xml.read().unwrap();
        match doc.root_element().unwrap().attribute(doc.deref(), "level") {
            Some(level) => Ok(level.parse().unwrap()),
            None => return Err("Required attribute \"level\" not specified.".to_string()),
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
            None => return Err("Required attribute \"version\" not specified.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{sbase::SBase, SbmlDocument};

    #[test]
    pub fn test_model_id() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.get_model();
        assert_eq!("model_id", model.get_id().unwrap().as_str());
        model.set_id("model_6431".to_string());
        assert_eq!("model_6431", model.get_id().unwrap().as_str());
        std::fs::write("test-inputs/model-modified.sbml", "dummy").unwrap();
        doc.write_path("test-inputs/model-modified.sbml").unwrap();
        let doc2 = SbmlDocument::read_path("test-inputs/model-modified.sbml").unwrap();
        let model2 = doc2.get_model();
        assert_eq!(model.get_id(), model2.get_id());
        assert_eq!(doc.to_xml_string(), doc2.to_xml_string());
        std::fs::remove_file("test-inputs/model-modified.sbml").unwrap();
    }
}
