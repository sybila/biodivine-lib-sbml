use crate::core::sbase::SbmlUtils;
use crate::xml::{OptionalProperty, RequiredProperty, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct QualitativeSpecies(XmlElement);

impl QualitativeSpecies {
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_qual_property("id")
    }

    pub fn name(&self) -> OptionalProperty<String> {
        self.optional_qual_property("name")
    }

    pub fn compartment(&self) -> RequiredProperty<String> {
        self.required_qual_property("compartment")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_qual_property("constant")
    }

    pub fn initial_level(&self) -> OptionalProperty<u32> {
        self.optional_qual_property("initialLevel")
    }

    pub fn max_level(&self) -> OptionalProperty<u32> {
        self.optional_qual_property("maxLevel")
    }
}
