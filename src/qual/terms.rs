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

/// The `<defaultTerm>` element of the Qual (Qualitative Models) package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "defaultTerm", namespace = NS_QUAL)]
pub struct DefaultTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for DefaultTerm {}

impl DefaultTerm {
    /// Creates a new `<defaultTerm>` element with the given `result_level`.
    pub fn new(document: Document, result_level: u32) -> DefaultTerm {
        let obj = DefaultTerm::new_empty(document);
        obj.result_level().set(&result_level);
        obj
    }

    /// The `resultLevel` attribute of this `<defaultTerm>` element.
    pub fn result_level(&self) -> RequiredSbmlProperty<'_, u32> {
        self.required_package_property("resultLevel", NS_QUAL, NS_QUAL)
    }
}

/// The `<functionTerm>` element of the Qual (Qualitative Models) package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "functionTerm", namespace = NS_QUAL)]
pub struct FunctionTerm(XmlElement);

impl XmlNamedSubtype<AbstractTerm> for FunctionTerm {}

impl FunctionTerm {
    /// Creates a new `<functionTerm>` element with the given `result_level`, `math`.
    pub fn new(document: Document, result_level: u32, math: Math) -> FunctionTerm {
        let obj = FunctionTerm::new_empty(document);
        obj.result_level().set(&result_level);
        obj.math().set(math);
        obj
    }
    /// The `resultLevel` attribute of this `<functionTerm>` element.
    pub fn result_level(&self) -> RequiredSbmlProperty<'_, u32> {
        self.required_package_property("resultLevel", NS_QUAL, NS_QUAL)
    }

    /// The `math` child of this term.
    ///
    /// A `functionTerm` is the one formula site whose accessor is *not* optional. Two things are
    /// worth knowing about that:
    ///
    ///  * **What the validation layer checks** is how *many* `math` children the element has: a
    ///    second one is a uniqueness violation of the type check
    ///    (`UNIQUE_CHILDREN["functionTerm"]`), which is the cardinality rule `qual-20804`
    ///    describes.
    ///  * **No rule of this crate reports a term that has none.** The table row that would have to
    ///    do it (`REQUIRED_CHILDREN`) is keyed `functionTerms`, a name no element carries, and is one
    ///    of the known table exceptions owned by the validation work; whether a `functionTerm` must
    ///    carry a formula at all is a specification question (rule `qual-20804` says the element
    ///    "may contain exactly one" `math` element) which the typed layer does not decide.
    ///
    /// The non-optional accessor is therefore a design choice of this crate — it keeps the content
    /// models' asymmetry visible — and the panic below is its own contract, not something a document
    /// check protects the caller from. Use [`FunctionTerm::math_node`] for the typed form, which has
    /// the same contract.
    ///
    /// # Panics
    ///
    /// Reading the child ([`RequiredXmlChild::get`]) panics when the element has none.
    pub fn math(&self) -> RequiredChild<'_, Math> {
        RequiredChild::new(self.xml_element(), "math", NS_MATHML)
    }

    /// The MathML formula of this term as a typed node.
    ///
    /// This is the one formula site whose accessor is *not* optional; [`FunctionTerm::math`]
    /// explains what the validation layer does and does not check about that, and why the choice of
    /// a mandatory accessor is this crate's rather than the specification's.
    ///
    /// # Panics
    ///
    /// Panics when the term has no `math` child, exactly as [`FunctionTerm::math`].
    pub fn math_node(&self) -> MathNode {
        self.math().get().node()
    }
}
