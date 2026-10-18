use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::Unit;
use crate::xml::{Document, OptionalSbmlChild, XmlDefault, XmlElement, XmlList};
use embed_doc_image::embed_doc_image;
use sbml_macros::{SBase, XmlWrapper};

/// Individual unit definition
///
/// ![UML][sbml-unitdefinition]
#[embed_doc_image("sbml-unitdefinition", "docs-images/uml-unitdefinition.svg")]
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "unitDefinition", namespace = NS_SBML_CORE)]
pub struct UnitDefinition(XmlElement);

impl UnitDefinition {
    /// The `listOfUnits` child of this `<unitDefinition>` element, as a list of [`Unit`].
    pub fn units(&self) -> OptionalSbmlChild<'_, XmlList<Unit>> {
        self.optional_sbml_child("listOfUnits")
    }
}

impl XmlDefault for UnitDefinition {
    fn default(document: Document) -> Self {
        UnitDefinition::new_empty(document)
    }
}
