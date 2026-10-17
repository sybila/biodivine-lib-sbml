use crate::constants::namespaces::{NS_MATHML, NS_QUAL};
use crate::core::sbase::SbmlUtils;
use crate::math::{Math, MathNode};
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

    /// The `math` child of this term.
    ///
    /// # Panics
    ///
    /// Reading the child ([`RequiredXmlChild::get`]) panics when
    /// the element has none. A `functionTerm` without a formula is a document error (rule
    /// `qual-20804` constrains how many `math` children the element may have, and the validation
    /// layer reports a document that has more than one), and the typed layer deliberately keeps the
    /// child required: use [`FunctionTerm::math_node`] for the typed form, which has the same
    /// contract.
    pub fn math(&self) -> RequiredChild<Math> {
        RequiredChild::new(self.xml_element(), "math", NS_MATHML)
    }

    /// The MathML formula of this term as a typed node.
    ///
    /// This is the one formula site whose accessor is *not* optional — the asymmetry the content
    /// models have, and a deliberate design choice of this crate rather than a mandate of the
    /// specification: rule `qual-20804` limits the number of `math` children of a `functionTerm`
    /// (a `functionTerm` may contain exactly one), while a term without one is a document the
    /// validation layer reports through its own rules rather than through a missing child.
    ///
    /// # Panics
    ///
    /// Panics when the term has no `math` child, exactly as [`FunctionTerm::math`].
    pub fn math_node(&self) -> MathNode {
        self.math().get().node()
    }
}
