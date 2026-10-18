use crate::constants::namespaces::NS_FBC;
use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::Species;
use crate::xml::{OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

/// The `<species>` element of the SBML core specification.
#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "species", namespace = NS_SBML_CORE)]
pub struct FbcSpecies(XmlElement);

impl XmlNamedSubtype<Species> for FbcSpecies {}

impl FbcSpecies {
    /// The `charge` attribute of this `<species>` element.
    pub fn charge(&self) -> OptionalSbmlProperty<'_, u32> {
        self.optional_package_property("charge", NS_FBC, NS_FBC)
    }
    /// The `chemicalFormula` attribute of this `<species>` element.
    pub fn formula(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("chemicalFormula", NS_FBC, NS_FBC)
    }
}
