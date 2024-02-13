use crate::constants::namespaces::URL_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement, XmlWrapper};
use macros::{SBase, XmlWrapper};
use std::ops::Deref;
use xml_doc::{Document, Element};

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    /// Try to find an instance of a [FunctionDefinition] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as it appears in an SBML model (i.e. one of
    /// its transitive parents is a [Model] element). If this is not satisfied, the method
    /// returns `None`.
    pub fn for_child_element(doc: XmlDocument, child: &XmlElement) -> Option<Self> {
        let parent = {
            let read_doc = doc.read().unwrap();
            fn is_function_definition(doc: &Document, e: Element) -> bool {
                let name = e.name(doc);
                let Some(namespace) = e.namespace(doc) else {
                    return false;
                };

                name == "functionDefinition" && namespace == URL_SBML_CORE
            }

            let mut parent = child.raw_element();
            while !is_function_definition(read_doc.deref(), parent) {
                let Some(node) = parent.parent(read_doc.deref()) else {
                    return None;
                };
                parent = node;
            }
            parent
        };
        let model = XmlElement::new_raw(doc, parent);
        // Safe because we checked that the element has the correct tag name and namespace.
        Some(unsafe { FunctionDefinition::unchecked_cast(model) })
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

impl XmlDefault for FunctionDefinition {
    fn default(document: XmlDocument) -> Self {
        FunctionDefinition::new_empty(document, "functionDefinition")
    }
}
