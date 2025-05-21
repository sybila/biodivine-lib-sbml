use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlChild, OptionalSbmlProperty, RequiredSbmlChild, RequiredSbmlProperty,
    RequiredXmlProperty, XmlDocument, XmlElement, XmlList, XmlNamedSubtype, XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Association(XmlElement);

impl XmlSupertype for Association {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct GeneProductRef(XmlElement);

impl XmlNamedSubtype<Association> for GeneProductRef {
    fn expected_tag_name() -> &'static str {
        "geneProductRef"
    }
}

impl GeneProductRef {
    pub fn new(document: XmlDocument, gene_product: SId) -> Self {
        let obj = GeneProductRef::new_empty(document, "geneProductRef");
        obj.gene_product().set(&gene_product);
        obj
    }
    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn gene_product(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("geneProduct", NS_FBC, NS_FBC)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct And(XmlElement);

impl XmlNamedSubtype<Association> for And {
    fn expected_tag_name() -> &'static str {
        "and"
    }
}

impl And {
    pub fn and(&self) -> OptionalSbmlChild<XmlList<Association>> {
        self.optional_package_child("and", NS_FBC, false)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Or(XmlElement);

impl XmlNamedSubtype<Association> for Or {
    fn expected_tag_name() -> &'static str {
        "or"
    }
}

impl Or {
    pub fn or(&self) -> RequiredSbmlChild<XmlList<Association>> {
        self.required_package_child("or", NS_FBC, false)
    }
}
