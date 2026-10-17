use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::math::{Math, MathNode};
use crate::xml::{
    Document, OptionalChild, OptionalXmlChild, RequiredSbmlProperty, RequiredXmlProperty,
    XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "initialAssignment", namespace = NS_SBML_CORE)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn new(document: Document, symbol: &SId) -> Self {
        let obj = InitialAssignment::new_empty(document);
        obj.symbol().set(symbol);
        obj
    }

    pub fn symbol(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }
}
