use crate::constants::namespaces::{NS_MATHML, NS_QUAL};
use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{
    RequiredChild, RequiredSbmlProperty, RequiredXmlChild, RequiredXmlProperty, XmlDocument,
    XmlElement, XmlNamedSubtype, XmlSupertype, XmlWrapper,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AbstractTerm(XmlElement);

impl XmlSupertype for AbstractTerm {}

impl AbstractTerm {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct DefaultTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for DefaultTerm {
    fn expected_tag_name() -> &'static str {
        "defaultTerm"
    }
}

impl DefaultTerm {
    pub fn new(document: XmlDocument, result_level: u32) -> DefaultTerm {
        let obj = DefaultTerm::new_empty(document, "defaultTerm");
        obj.result_level().set(&result_level);
        obj
    }

    pub fn result_level(&self) -> RequiredSbmlProperty<u32> {
        self.required_package_property("resultLevel", NS_QUAL, NS_QUAL)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for FunctionTerm {
    fn expected_tag_name() -> &'static str {
        "functionTerm"
    }
}

impl FunctionTerm {
    pub fn new(document: XmlDocument, result_level: u32, math: Math) -> FunctionTerm {
        let obj = FunctionTerm::new_empty(document, "functionTerm");
        obj.result_level().set(&result_level);
        obj.math().set(math);
        obj
    }
    pub fn result_level(&self) -> RequiredSbmlProperty<u32> {
        self.required_package_property("resultLevel", NS_QUAL, NS_QUAL)
    }

    pub fn math(&self) -> RequiredChild<Math> {
        RequiredChild::new(self.xml_element(), "math", NS_MATHML)
    }
}
