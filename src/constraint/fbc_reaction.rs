use sbml_macros::{SBase, XmlWrapper};
use crate::constants::namespaces::NS_FBC;
use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::{Reaction, SId};
use crate::core::sbase::SbmlUtils;
use crate::xml::{OptionalChild, OptionalSbmlProperty, XmlElement, XmlNamedSubtype, XmlSubtype};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct FbcReaction(XmlElement);

impl XmlNamedSubtype<Reaction> for FbcReaction {
    fn expected_tag_name() -> &'static str {
        "reaction"
    }
}

impl FbcReaction {
    pub fn lowerFluxBound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("lowerFluxBound", NS_FBC, NS_FBC)
    }
    pub fn upperFluxBound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("upperFluxBound", NS_FBC, NS_FBC)
    }
    pub fn geneProductAssociation(&self) -> OptionalChild<GeneProductAssociation> {
        self.optional_package_child("geneProductAssociation", NS_FBC, false)
    }
}