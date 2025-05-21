use crate::constants::namespaces::NS_FBC;
use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::sbase::SbmlUtils;
use crate::core::{Reaction, SId};
use crate::xml::{OptionalChild, OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct FbcReaction(XmlElement);

impl XmlNamedSubtype<Reaction> for FbcReaction {
    fn expected_tag_name() -> &'static str {
        "reaction"
    }
}

impl FbcReaction {
    pub fn lower_flux_bound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("lowerFluxBound", NS_FBC, NS_FBC)
    }
    pub fn upper_flux_bound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("upperFluxBound", NS_FBC, NS_FBC)
    }
    pub fn gene_product_association(&self) -> OptionalChild<GeneProductAssociation> {
        self.optional_package_child("geneProductAssociation", NS_FBC, false)
    }
}
