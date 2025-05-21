use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct GeneProduct(XmlElement);

impl GeneProduct {
    pub fn new(document: XmlDocument, id: SId, label: String) -> GeneProduct {
        let obj = GeneProduct::new_empty(document, "geneProduct");
        obj.id().set(&id);
        obj.label().set(&label);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn label(&self) -> RequiredSbmlProperty<String> {
        self.required_package_property("label", NS_FBC, NS_FBC)
    }
    pub fn associated_species(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("associatedSpecies", NS_FBC, NS_FBC)
    }
}
