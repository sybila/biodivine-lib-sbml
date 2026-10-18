use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlChild, OptionalSbmlProperty, RequiredSbmlChild, RequiredSbmlProperty,
    RequiredXmlProperty, XmlElement, XmlList, XmlNamedSubtype, XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};

/// An operand of a gene product association: a `geneProductRef`, an `and` or an `or`.
///
/// The first of the three names belongs to the concrete [`GeneProductRef`], so the identity-derived
/// constructor is suppressed: `new_empty(document)` here would build a `<geneProductRef>`, which is
/// what [`GeneProductRef::new_empty`] is for. A nested expression is built from [`And`]/[`Or`] and
/// read back through this supertype with `try_cast_from_super`.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(names = ["geneProductRef", "and", "or"], namespace = NS_FBC, constructor = false)]
pub struct Association(XmlElement);

impl XmlSupertype for Association {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "geneProductRef", namespace = NS_FBC)]
pub struct GeneProductRef(XmlElement);

impl XmlNamedSubtype<Association> for GeneProductRef {}

impl GeneProductRef {
    pub fn new(document: Document, gene_product: SId) -> Self {
        let obj = GeneProductRef::new_empty(document);
        obj.gene_product().set(&gene_product);
        obj
    }
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn gene_product(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("geneProduct", NS_FBC, NS_FBC)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "and", namespace = NS_FBC)]
pub struct And(XmlElement);

impl XmlNamedSubtype<Association> for And {}

impl And {
    pub fn and(&self) -> OptionalSbmlChild<'_, XmlList<Association>> {
        self.optional_package_child("and", NS_FBC, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "or", namespace = NS_FBC)]
pub struct Or(XmlElement);

impl XmlNamedSubtype<Association> for Or {}

impl Or {
    pub fn or(&self) -> RequiredSbmlChild<'_, XmlList<Association>> {
        self.required_package_child("or", NS_FBC, false)
    }
}
