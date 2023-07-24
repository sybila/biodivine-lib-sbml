use crate::xml::{XmlDocument, XmlElement, XmlList, XmlWrapper};
use crate::sbase::{SBase, SBaseDefault};
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
    xmlns: String,
    level: u32,
    version: u32,
}

/// Representation of all optional unit definitions + conversion factor of SBML model
#[derive(Clone, Debug, Default)]
struct SbmlModelUnits {
    // use enums of recommended units + functions to map values to string ?
    substance_units: Option<String>,
    time_units: Option<String>,
    volume_units: Option<String>,
    area_units: Option<String>,
    length_units: Option<String>,
    extent_units: Option<String>,
    conversion_factor: Option<String>, // use of enum also possible here ?
}

/// Representation of all optional list of SBML model
#[derive(Clone, Debug, Default)]
struct SbmlModelLists {
    function_definitions: Option<Vec<()>>, // TODO: define type for individial function def
    unit_definitions: Option<Vec<()>>,     // TODO: define type for individual unit def
    compartments: Option<Vec<()>>,         // TODO: define type for individual compartment
    species: Option<Vec<()>>,              // TODO: define type for individial specie
    parameters: Option<Vec<()>>,           // TODO: define type for individual parameter
    initial_assignments: Option<Vec<()>>,  // TODO: define type for individual initial assignment
    rules: Option<Vec<()>>,                // TODO: define type for individual rule
    constraints: Option<Vec<()>>,          // TODO: define type for individual constraint
    reactions: Option<Vec<()>>,            // TODO: define type for individual reaction
    events: Option<Vec<()>>,               // TODO: define type for individual event
}

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug)]
pub struct SbmlModel {
    xml: XmlElement,
    units: SbmlModelUnits,
    lists: SbmlModelLists,
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

/// TODO: If I recall correctly, these should also implement SBase, but remove if that's not true.
impl SBaseDefault for SbmlFunctionDefinition {}

impl SbmlModel {
    pub fn get_function_definitions(&self) -> XmlList<SbmlFunctionDefinition> {
        let list_element = {
            let xml = self.read_doc();
            self.element()
                .find(xml.deref(), "ListOfFunctionDefinitions")
                .unwrap()
        };
        XmlList::from(self.as_xml().derive(list_element))
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
        let sbml_element = match doc.root_element() {
            None => return Err("No root <sbml> element present.".to_string()),
            Some(element) => element,
        };
        let xmlns = match sbml_element.namespace_decls(&doc).get("") {
            Some(xmlns) => xmlns.to_string(),
            None => {
                return Err("No xmlns namespace attribute present in <sbml> element".to_string())
            }
        };
        let level: u32 = match sbml_element.attribute(&doc, "level") {
            None => {
                return Err("<sbml> element does not contain info about level used.".to_string())
            }
            Some(level) => match level.parse() {
                Ok(number) => number,
                Err(why) => return Err(why.to_string()), // more specific error message needed ?
            },
        };
        let version: u32 = match sbml_element.attribute(&doc, "version") {
            None => {
                return Err("<sbml> element does not contain info about version used.".to_string())
            }
            Some(version) => match version.parse() {
                Ok(number) => number,
                Err(why) => return Err(why.to_string()), // more specific error message needed ?
            },
        };

        Ok(SbmlDocument {
            xml: Arc::new(RwLock::new(doc)),
            xmlns,
            level,
            version,
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
            units: Default::default(),
            lists: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SBase, SbmlDocument};

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
