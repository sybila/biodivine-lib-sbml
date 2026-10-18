use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::math::{Math, MathNode};
use crate::xml::{Document, OptionalChild, OptionalXmlChild, XmlDefault, XmlElement};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "constraint", namespace = NS_SBML_CORE)]
pub struct Constraint(XmlElement);

impl XmlDefault for Constraint {
    fn default(document: Document) -> Self {
        Constraint::new_empty(document)
    }
}

impl Constraint {
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

    pub fn message(&self) -> OptionalChild<'_, XmlElement> {
        self.optional_html_child("message")
    }
}
