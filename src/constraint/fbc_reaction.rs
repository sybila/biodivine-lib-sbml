use crate::constants::namespaces::NS_FBC;
use crate::constants::namespaces::NS_SBML_CORE;
use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::sbase::SbmlUtils;
use crate::core::{Reaction, SId};
use crate::xml::{OptionalSbmlChild, OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "reaction", namespace = NS_SBML_CORE)]
pub struct FbcReaction(XmlElement);

impl XmlNamedSubtype<Reaction> for FbcReaction {}

impl FbcReaction {
    pub fn lower_flux_bound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("lowerFluxBound", NS_FBC, NS_FBC)
    }
    pub fn upper_flux_bound(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("upperFluxBound", NS_FBC, NS_FBC)
    }
    pub fn gene_product_association(&self) -> OptionalSbmlChild<GeneProductAssociation> {
        self.optional_package_child("geneProductAssociation", NS_FBC, false)
    }
}
