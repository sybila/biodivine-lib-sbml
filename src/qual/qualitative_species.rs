use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// The `<qualitativeSpecies>` element of the Qual (Qualitative Models) package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "qualitativeSpecies", namespace = NS_QUAL)]
pub struct QualitativeSpecies(XmlElement);

impl QualitativeSpecies {
    /// Creates a new `<qualitativeSpecies>` element with the given `id`, `compartment`, `constant`.
    pub fn new(
        document: Document,
        id: SId,
        compartment: SId,
        constant: bool,
    ) -> QualitativeSpecies {
        let obj = QualitativeSpecies::new_empty(document);
        obj.id().set(&id);
        obj.compartment().set(&compartment);
        obj.constant().set(&constant);
        obj
    }
    /// The `id` attribute of this `<qualitativeSpecies>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_QUAL, NS_QUAL)
    }
    /// The `name` attribute of this `<qualitativeSpecies>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_QUAL, NS_QUAL)
    }
    /// The `compartment` attribute of this `<qualitativeSpecies>` element.
    pub fn compartment(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("compartment", NS_QUAL, NS_QUAL)
    }

    /// The `constant` attribute of this `<qualitativeSpecies>` element.
    pub fn constant(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_package_property("constant", NS_QUAL, NS_QUAL)
    }

    /// The `initialLevel` attribute of this `<qualitativeSpecies>` element.
    pub fn initial_level(&self) -> OptionalSbmlProperty<'_, u32> {
        self.optional_package_property("initialLevel", NS_QUAL, NS_QUAL)
    }

    /// The `maxLevel` attribute of this `<qualitativeSpecies>` element.
    pub fn max_level(&self) -> OptionalSbmlProperty<'_, u32> {
        self.optional_package_property("maxLevel", NS_QUAL, NS_QUAL)
    }
}
