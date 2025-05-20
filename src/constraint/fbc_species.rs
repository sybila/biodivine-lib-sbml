use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::Species;
use crate::xml::{OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct FbcSpecies(XmlElement);

impl XmlNamedSubtype<Species> for FbcSpecies {
    fn expected_tag_name() -> &'static str {
        "species"
    }
}

impl FbcSpecies {
    pub fn charge(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("charge", NS_FBC, NS_FBC)
    }
    pub fn formula(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("chemicalFormula", NS_FBC, NS_FBC)
    }
}
