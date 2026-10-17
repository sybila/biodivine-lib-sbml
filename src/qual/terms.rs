use crate::constants::namespaces::{NS_MATHML, NS_QUAL};
use crate::core::sbase::SbmlUtils;
use crate::core::Math;
use crate::xml::{
    Document, RequiredChild, RequiredSbmlProperty, RequiredXmlChild, RequiredXmlProperty,
    XmlElement, XmlNamedSubtype, XmlSupertype, XmlWrapper,
};
use sbml_macros::{SBase, XmlWrapper};

/// A term of a qualitative transition: a `defaultTerm` or a `functionTerm`.
///
/// The first of the two names belongs to the concrete [`DefaultTerm`], so the identity-derived
/// constructor is suppressed: `new_empty(document)` here would build a `<defaultTerm>`, which is
/// what [`DefaultTerm::new_empty`] is for. A term of a known kind is built through the concrete type
/// and read back through this supertype with `try_cast_from_super`.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(names = ["defaultTerm", "functionTerm"], namespace = NS_QUAL, constructor = false)]
pub struct AbstractTerm(XmlElement);

impl XmlSupertype for AbstractTerm {}

impl AbstractTerm {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "defaultTerm", namespace = NS_QUAL)]
pub struct DefaultTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for DefaultTerm {}

impl DefaultTerm {
    pub fn new(document: Document, result_level: u32) -> DefaultTerm {
        let obj = DefaultTerm::new_empty(document);
        obj.result_level().set(&result_level);
        obj
    }

    pub fn result_level(&self) -> RequiredSbmlProperty<u32> {
        self.required_package_property("resultLevel", NS_QUAL, NS_QUAL)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "functionTerm", namespace = NS_QUAL)]
pub struct FunctionTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for FunctionTerm {}

impl FunctionTerm {
    pub fn new(document: Document, result_level: u32, math: Math) -> FunctionTerm {
        let obj = FunctionTerm::new_empty(document);
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
