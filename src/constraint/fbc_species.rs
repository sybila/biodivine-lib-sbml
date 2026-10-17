use crate::constants::namespaces::NS_FBC;
use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::Species;
use crate::xml::{OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "species", namespace = NS_SBML_CORE)]
pub struct FbcSpecies(XmlElement);

impl XmlNamedSubtype<Species> for FbcSpecies {}

impl FbcSpecies {
    pub fn charge(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("charge", NS_FBC, NS_FBC)
    }
    pub fn formula(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("chemicalFormula", NS_FBC, NS_FBC)
    }
}
