use crate::constants::namespaces::NS_FBC;
use crate::constraint::association::{Association, GeneProductRef};
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{OptionalSbmlChild, OptionalSbmlProperty, XmlElement, XmlList};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "geneProductAssociation", namespace = NS_FBC)]
pub struct GeneProductAssociation(XmlElement);

impl GeneProductAssociation {
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn or(&self) -> OptionalSbmlChild<'_, XmlList<Association>> {
        self.optional_package_child("or", NS_FBC, false)
    }
    pub fn and(&self) -> OptionalSbmlChild<'_, XmlList<Association>> {
        self.optional_package_child("and", NS_FBC, false)
    }
    pub fn gene_product_ref(&self) -> OptionalSbmlChild<'_, GeneProductRef> {
        self.optional_package_child("geneProductRef", NS_FBC, false)
    }
}
