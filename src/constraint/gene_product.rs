use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// The `<geneProduct>` element of the FBC (Flux Balance Constraints) package specification.
#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "geneProduct", namespace = NS_FBC)]
pub struct GeneProduct(XmlElement);

impl GeneProduct {
    /// Creates a new `<geneProduct>` element with the given `id`, `label`.
    pub fn new(document: Document, id: SId, label: String) -> GeneProduct {
        let obj = GeneProduct::new_empty(document);
        obj.id().set(&id);
        obj.label().set(&label);
        obj
    }

    /// The `id` attribute of this `<geneProduct>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_FBC, NS_FBC)
    }
    /// The `name` attribute of this `<geneProduct>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    /// The `label` attribute of this `<geneProduct>` element.
    pub fn label(&self) -> RequiredSbmlProperty<'_, String> {
        self.required_package_property("label", NS_FBC, NS_FBC)
    }
    /// The `associatedSpecies` attribute of this `<geneProduct>` element.
    pub fn associated_species(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("associatedSpecies", NS_FBC, NS_FBC)
    }
}
