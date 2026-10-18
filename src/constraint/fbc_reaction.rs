use crate::constants::namespaces::NS_FBC;
use crate::constants::namespaces::NS_SBML_CORE;
use crate::constraint::gene_product_association::GeneProductAssociation;
use crate::core::sbase::SbmlUtils;
use crate::core::{Reaction, SId};
use crate::xml::{OptionalSbmlChild, OptionalSbmlProperty, XmlElement, XmlNamedSubtype};
use sbml_macros::{SBase, XmlWrapper};

/// The `<reaction>` element of the SBML core specification.
#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "reaction", namespace = NS_SBML_CORE)]
pub struct FbcReaction(XmlElement);

impl XmlNamedSubtype<Reaction> for FbcReaction {}

impl FbcReaction {
    /// The `lowerFluxBound` attribute of this `<reaction>` element.
    pub fn lower_flux_bound(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("lowerFluxBound", NS_FBC, NS_FBC)
    }
    /// The `upperFluxBound` attribute of this `<reaction>` element.
    pub fn upper_flux_bound(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("upperFluxBound", NS_FBC, NS_FBC)
    }
    /// The `geneProductAssociation` child of this `<reaction>` element.
    pub fn gene_product_association(&self) -> OptionalSbmlChild<'_, GeneProductAssociation> {
        self.optional_package_child("geneProductAssociation", NS_FBC, false)
    }
}
