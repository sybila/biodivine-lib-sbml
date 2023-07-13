use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use xml_doc::{Document, Element};

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
type XmlDocument = Arc<RwLock<Document>>;

/// The object that "wraps" an XML document in a SBML-specific API.
///
/// This is mostly just the place where you can specify what SBML version and
/// what SBML extensions are being used. The actual content of the SBML model is
/// then managed through the `SbmlModel` struct.
#[derive(Clone, Debug)]
pub struct SbmlDocument {
    xml: XmlDocument,
}

/// A type-safe representation of an SBML `model` element.
#[derive(Clone, Debug)]
pub struct SbmlModel {
    xml: XmlDocument,
    element: Element,
}

impl SbmlDocument {
    pub fn read_path(path: &str) -> SbmlDocument {
        // TODO: Error handling
        let file_contents = std::fs::read_to_string(path).unwrap();
        let doc = Document::from_str(file_contents.as_str()).unwrap();
        SbmlDocument {
            xml: Arc::new(RwLock::new(doc)),
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
            xml: self.xml.clone(),
            element: model_element,
        }
    }
}

impl SbmlModel {
    pub fn get_id(&self) -> String {
        let xml = self.xml.read().unwrap();
        // Unfortunately, here the reference returned by the `attribute` function is only
        // valid for as long as the `xml` document is locked. Hence we can't return it,
        // because once this function completes, the lock is released and the reference becomes
        // unsafe to access. We thus have to copy the contents of the string using `to_string`.
        self.element
            .attribute(xml.deref(), "id")
            .unwrap()
            .to_string()
    }

    pub fn set_id(&self, value: &str) {
        // Here, we are locking for writing. Note that we don't need a `&mut self` reference for
        // this, because this "bypasses" normal borrow checker rules. So we can safely create
        // a mutable reference and the caller of this function does not have to care if his
        // reference to the document is mutable or not (that's the primary purpose of the RwLock,
        // because to pass the borrow checker rules, all data shared between threads must be
        // "read only"; in this way, we can make a "read only" object that has internal mutability).
        let mut xml = self.xml.write().unwrap();
        self.element.set_attribute(xml.deref_mut(), "id", value);
    }
}

#[cfg(test)]
mod tests {
    use crate::SbmlDocument;

    #[test]
    pub fn test_model_id() {
        let doc = SbmlDocument::read_path("test-inputs/model.sbml");
        let model = doc.get_model();
        assert_eq!("model_id", model.get_id().as_str());
        model.set_id("model_6431");
        assert_eq!("model_6431", model.get_id().as_str());
    }
}
