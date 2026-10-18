use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::math::{Math, MathNode};
use crate::xml::{Document, OptionalChild, OptionalXmlChild, XmlDefault, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "functionDefinition", namespace = NS_SBML_CORE)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    /// Try to find an instance of a [FunctionDefinition] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as one of its transitive parents is a
    /// [FunctionDefinition] element. If this is not satisfied, the method returns `None`.
    pub fn for_child_element(child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(child, "functionDefinition")
    }

    /// The `math` child of this `<functionDefinition>` element.
    pub fn math(&self) -> OptionalChild<'_, Math> {
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

impl XmlDefault for FunctionDefinition {
    fn default(document: Document) -> Self {
        FunctionDefinition::new_empty(document)
    }
}
