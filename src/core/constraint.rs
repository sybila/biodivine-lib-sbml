use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::math::{Math, MathNode};
use crate::xml::{Document, OptionalChild, OptionalXmlChild, XmlDefault, XmlElement};
use embed_doc_image::embed_doc_image;
use sbml_macros::{SBase, XmlWrapper};

/// The `<constraint>` element of the SBML core specification.
///
/// ![UML][sbml-constraint]
#[embed_doc_image("sbml-constraint", "docs-images/uml-constraint.svg")]
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "constraint", namespace = NS_SBML_CORE)]
pub struct Constraint(XmlElement);

impl XmlDefault for Constraint {
    fn default(document: Document) -> Self {
        Constraint::new_empty(document)
    }
}

impl Constraint {
    /// The `math` child of this `<constraint>` element.
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

    /// The `message` child of this `<constraint>` element.
    pub fn message(&self) -> OptionalChild<'_, XmlElement> {
        self.optional_html_child("message")
    }
}
