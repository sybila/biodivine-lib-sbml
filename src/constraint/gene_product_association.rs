use sbml_macros::{SBase, XmlWrapper};
use crate::constants::namespaces::NS_FBC;
use crate::constraint::association::{Association, GeneProductRef};
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{OptionalChild, OptionalSbmlProperty, RequiredChild, XmlElement, XmlList};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct GeneProductAssociation(XmlElement);

impl GeneProductAssociation {
    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn or(&self) -> OptionalChild<XmlList<Association>> {
        self.optional_package_child("or", NS_FBC, false)
    }
    pub fn and(&self) -> OptionalChild<XmlList<Association>> {
        self.optional_package_child("and", NS_FBC, false)
    }
    pub fn geneProductRef(&self) -> OptionalChild<GeneProductRef> {
        self.optional_package_child("geneProductRef", NS_FBC, false)
    }
}