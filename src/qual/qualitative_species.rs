use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct QualitativeSpecies(XmlElement);

impl QualitativeSpecies {
    pub fn new(
        document: XmlDocument,
        id: SId,
        compartment: SId,
        constant: bool,
    ) -> QualitativeSpecies {
        let obj = QualitativeSpecies::new_empty(document, "qualitativeSpecies");
        obj.id().set(&id);
        obj.compartment().set(&compartment);
        obj.constant().set(&constant);
        obj
    }
    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_QUAL, NS_QUAL)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_QUAL, NS_QUAL)
    }
    pub fn compartment(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("compartment", NS_QUAL, NS_QUAL)
    }

    pub fn constant(&self) -> RequiredSbmlProperty<bool> {
        self.required_package_property("constant", NS_QUAL, NS_QUAL)
    }

    pub fn initial_level(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("initialLevel", NS_QUAL, NS_QUAL)
    }

    pub fn max_level(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("maxLevel", NS_QUAL, NS_QUAL)
    }
}
