//! The MathML subset that SBML uses, as a typed layer.
//!
//! SBML does not define formulas itself: wherever a model needs an expression — the rate of a
//! reaction, the value of a parameter, the condition of an event — it contains a `math` element of
//! the **MathML** namespace, holding an expression of the MathML subset that the SBML specification
//! fixes. This module is that subset, typed: a formula can be read, navigated, built and edited
//! through Rust types instead of through element names.
//!
//! ```rust
//! use biodivine_lib_sbml::core::*;
//! use biodivine_lib_sbml::math::*;
//! use biodivine_lib_sbml::xml::{OptionalXmlChild, XmlWrapper};
//! use biodivine_lib_sbml::Sbml;
//!
//! let document = Sbml::read_str(
//!     r#"<?xml version="1.0"?>
//!        <sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
//!          <model id="m"><listOfReactions><reaction id="r">
//!            <kineticLaw><math xmlns="http://www.w3.org/1998/Math/MathML">
//!              <apply><times/><ci>k1</ci><ci>S1</ci></apply>
//!            </math></kineticLaw>
//!          </reaction></listOfReactions></model>
//!        </sbml>"#,
//! )
//! .expect("The document is well formed.");
//! let model = document.model().get().expect("The document has a model.");
//!
//! // A formula site of the object model gives the `math` element, and `MathNode` is a tree:
//! // `children` gives the typed child nodes and `parent` walks back up.
//! let formula = model
//!     .reactions()
//!     .get()
//!     .expect("The model has reactions.")
//!     .get(0)
//!     .kinetic_law()
//!     .get()
//!     .expect("The reaction has a kinetic law.")
//!     .math()
//!     .get()
//!     .expect("Its kinetic law has a formula.");
//! let node = formula.node();
//! assert_eq!(node.element_name(), "math");
//! let expression = node.children().into_iter().next().expect("The formula is not empty.");
//! assert_eq!(expression.parent().unwrap().element_name(), "math");
//!
//! // A formula can also be built from scratch.
//! let handle = document.xml_element().document();
//! let built = Math::new_empty(handle.clone());
//! built.set_expression(MathNode::apply(
//!     handle.clone(),
//!     MathOperator::Divide,
//!     [
//!         MathNode::ci(handle.clone(), "k1"),
//!         MathNode::integer(handle, 2),
//!     ],
//! ));
//! assert_eq!(built.expression().unwrap().element_name(), "apply");
//! ```
//!
//! # What the subset contains
//!
//! The subset is the list of elements that rule 10202 of the specification permits inside a `math`
//! element, plus the `math` element itself: **73 names**, which split into three groups.
//!
//! **15 structural elements** have a structure of their own, so each is a declared type in this
//! module, through the same [`#[sbml_object]`](crate::xml::SbmlObject) declaration the SBML objects
//! use:
//!
//! | element | content model (the order the builders write) |
//! | --- | --- |
//! | `apply` | the operator, then the arguments — [`Apply`](crate::math::Apply) |
//! | `bvar` | one `ci` — [`BVar`](crate::math::BVar) |
//! | `ci` | an identifier as text — [`Ci`](crate::math::Ci) |
//! | `cn` | a number as text, optionally with a `sep` between two tokens — [`Cn`](crate::math::Cn) |
//! | `csymbol` | a symbol identified by its `definitionURL` — [`Csymbol`](crate::math::Csymbol) |
//! | `degree` | one expression, the degree of a `root` — [`Degree`](crate::math::Degree) |
//! | `lambda` | the bound variables, then the body — [`Lambda`](crate::math::Lambda) |
//! | `logbase` | one expression, the base of a `log` — [`Logbase`](crate::math::Logbase) |
//! | `otherwise` | one expression, the default of a `piecewise` — [`Otherwise`](crate::math::Otherwise) |
//! | `piece` | a value, then its condition — [`Piece`](crate::math::Piece) |
//! | `piecewise` | the cases, then the default — [`Piecewise`](crate::math::Piecewise) |
//! | `semantics` | the expression, then the annotations — [`Semantics`](crate::math::Semantics) |
//! | `sep` | nothing; the separator of a two-token number — [`Sep`](crate::math::Sep) |
//! | `annotation` | text plus an `encoding` — [`Annotation`](crate::math::Annotation) |
//! | `annotation-xml` | arbitrary markup plus an `encoding` — [`AnnotationXml`](crate::math::AnnotationXml) |
//!
//! **52 operators** and **6 constants** have no content at all: their meaning is their name, and the
//! specification fixes the set. They are therefore values of two closed enumerations,
//! [`MathOperator`](crate::math::MathOperator) and [`MathConstant`](crate::math::MathConstant), rather than declared types. The operators carry the
//! argument count rule 10218 accepts for them ([`MathOperator::arity`](crate::math::MathOperator::arity));
//!
//! * 33 of them have a unary entry in the specification's operator tables (32 that are unary only,
//!   plus `minus`, which is unary *and* binary);
//! * 7 have a binary entry (`minus`, `root`, `divide`, `power`, `quotient`, `rem`, `implies`);
//! * the remaining 13 (`and`, `eq`, `geq`, `gt`, `leq`, `lt`, `max`, `min`, `neq`, `or`, `plus`,
//!   `times`, `xor`) are in neither table — the operators whose argument count rule 10218 does not
//!   constrain, because MathML defines them as N-ary.
//!
//! [`MathNode`](crate::math::MathNode) is the sum type over all of them: it is what a formula is a tree of, and
//! [`MathNode::try_from_element`](crate::math::MathNode::try_from_element) is the conversion that decides whether an element is part of the
//! subset.
//!
//! # Where the list comes from
//!
//! The layer follows the **crate's validation constants** (`MATHML_ALLOWED_CHILDREN` and its
//! siblings in `src/constants/element.rs`), which are the list the validation layer enforces
//! (rules 10202 to 10225, in `src/core/validation/math.rs`), and which agree exactly with the
//! normative rule 10202 of the specification and with the prose list of Section 3.4.1. The XML
//! Schema shipped in the specification repository is five names short of that list (`implies`,
//! `max`, `min`, `quotient` and `rem`), and it is the schema that is stale: it describes the
//! Level 2 subset, while the five missing names are the ones the Level 3 Version 2 specification
//! adds (they are marked as version-2 additions both in rule 10202 and in the prose). A test pins
//! the typed layer against the validation constants in both directions, so the two cannot drift
//! apart; an element the constants do not permit is outside the subset here as well.
//!
//! # Three things this module does not do
//!
//!  * **It does not validate.** A formula that breaks a rule is still readable, editable and
//!    writable through this layer; the rules are checked by the validation layer, which reports
//!    them like any other SBML issue. The typed accessors are deliberately permissive for that
//!    reason: a caller has to be able to *repair* a document.
//!  * **It does not interpret numbers.** The payload of a `cn` is a token sequence, not a number;
//!    its numeric meaning depends on the `type` attribute, on the model's units and on the MathML
//!    semantics. [`Cn::payload`](crate::math::Cn::payload) returns what the document says, and it is the consumer's job to
//!    interpret it.
//!  * **It does not cover the whole of MathML.** The subset is what SBML permits; everything else
//!    (an element of another version of MathML, a construct of a future revision) is outside the
//!    *typed* layer. That is a type verdict, not a limitation of the reader: the element and its
//!    content stay reachable, unchanged, through the raw element of its parent
//!    ([`MathNode::raw_children`](crate::math::MathNode::raw_children) gives every child element of
//!    a node and [`MathNode::xml_element`](crate::math::MathNode::xml_element) the element itself),
//!    and a document containing such an element round trips through the library like any other.
//!    There is no scope-out list to keep in sync, because the typed layer accepts every name the
//!    validation accepts. One operation *is* destructive: replacing the content of a formula
//!    ([`Math::set_expression`](crate::math::Math::set_expression) or
//!    [`MathNode::set_children`](crate::math::MathNode::set_children)) detaches every child element
//!    of the replaced region, including one this layer cannot describe, so a caller that wants to
//!    keep such a child edits through the raw element instead.
//!
//! # Namespaces
//!
//! MathML is a namespace of its own (`http://www.w3.org/1998/Math/MathML`), and a `math` element is
//! the one place in an SBML document where that namespace appears. Following the XML layer of this
//! crate, this module never rewrites the declarations of a document it reads or edits the
//! expression of; it declares the namespace where a *new* formula needs it, and a formula built
//! from scratch carries exactly one declaration, on its `math` element
//! ([`Math::new_empty`](crate::math::Math::new_empty) writes it, the construction helpers do not repeat it on every node, and
//! [`MathNode::push`](crate::math::MathNode::push) adds one only where a node would otherwise inherit the wrong namespace).
//!
//! The SBML `units` attribute of a `cn` element is the one attribute of a MathML element that lives
//! in the *SBML* namespace, so it has to be written with a prefix (`sbml:units`); see [`SbmlUnits`](crate::math::SbmlUnits).
//!
//! # Acknowledgment
//!
//! The element set, the content models, the operator tables and the wording of the rule references
//! in this module are adapted from the *Systems Biology Markup Language (SBML) Level 3 Version 2
//! Core* specification and from *MathML 2.0* (a W3C recommendation), as distributed by the SBML
//! Team. The SBML specification is authored by Michael Hucka, Frank T. Bergmann, Claudine Chaouiya,
//! Andreas Dräger, Stefan Hoops, Sarah M. Keating, Matthias König, Nicolas Le Novère, Chris J.
//! Myers, Brett G. Olivier, Sven Sahle, James C. Schaff, Lucian P. Smith, Fengkai Zhang and others,
//! and is made available by the California Institute of Technology, the University of Heidelberg,
//! the University of Washington and the SBML Team under the terms of the Creative Commons
//! Attribution-ShareAlike license.
//!
//! # Where to go next
//!
//!  * [`Math`](crate::math::Math) — the formula root, and how to attach or replace a formula.
//!  * [`MathNode`](crate::math::MathNode) — navigation ([`MathNode::children`](crate::math::MathNode::children), [`MathNode::parent`](crate::math::MathNode::parent)) and rendering.
//!  * [`MathNode::apply`](crate::math::MathNode::apply) and its neighbours — building a formula, in the element order the
//!    specification fixes.
//!  * The concrete element types (for example [`Apply`](crate::math::Apply), [`Cn`](crate::math::Cn), [`Lambda`](crate::math::Lambda)) — the accessors of a
//!    node's content.

mod elements;
mod node;
mod operator;
mod units;

use crate::constants::namespaces::NS_MATHML;
use crate::xml::{Document, XmlDefault, XmlElement, XmlWrapper};
use sbml_macros::XmlWrapper;
use std::fmt::{Display, Formatter};

pub use elements::{
    Annotation, AnnotationXml, Apply, BVar, Ci, Cn, CnType, Csymbol, Degree, Lambda, Logbase,
    Otherwise, Piece, Piecewise, Semantics, Sep,
};
pub use node::{MathNode, MathNodeKind};
pub use operator::{MathArity, MathConstant, MathOperator, MathSymbol};
pub use units::SbmlUnits;

/// The MathML `math` element: the root of a formula.
///
/// Wherever an SBML object needs an expression, it has an optional (or, for the qualitative-model
/// `functionTerm`, required) child of this type; the typed accessors of those objects
/// (`KineticLaw::math_node`, `Constraint::math_node`, …) return the root as a [`MathNode`], which is
/// what a caller navigates.
///
/// The element belongs to the MathML namespace, following rule 10201. It is the only node of a
/// formula that declares that namespace: a formula built through the construction helpers of
/// [`MathNode`] carries the declaration here and nowhere else.
///
/// The content model of a `math` element is a single expression (a `lambda` inside a
/// `functionDefinition`, everything else anywhere). This type does not insist on that: a document
/// with no expression, or with several, is still readable and editable, and the validation layer
/// reports what is wrong with it.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "math", namespace = NS_MATHML)]
pub struct Math(XmlElement);

impl Math {
    /// A new formula in `document` whose single expression is `expression`.
    ///
    /// This is the one-step form of [`Math::new_empty`] followed by [`Math::set_expression`], which
    /// is what attaching a formula to one of the object model's formula sites looks like:
    /// `kinetic_law.math().set(Math::new(document, node))`.
    ///
    /// ```rust
    /// use biodivine_lib_sbml::math::*;
    /// use biodivine_lib_sbml::xml::Document;
    ///
    /// let document = Document::parse_string("<?xml version=\"1.0\"?><probe/>")
    ///     .expect("A well-formed document.");
    /// let formula = Math::new(document.clone(), MathNode::ci(document, "x"));
    /// assert_eq!(
    ///     formula.to_string(),
    ///     "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><ci>x</ci></math>"
    /// );
    /// ```
    pub fn new(document: Document, expression: MathNode) -> Math {
        let formula = Math::new_empty(document);
        formula.set_expression(expression);
        formula
    }

    /// This formula as a typed node.
    ///
    /// The node is the root of the tree a caller navigates; it costs nothing, because it holds the
    /// element rather than a copy of it.
    pub fn node(&self) -> MathNode {
        MathNode::Math(self.clone())
    }

    /// The top-level expressions of this formula, as typed nodes.
    ///
    /// A valid `math` element has exactly one; a document with several, or with none, is reported by
    /// the validation layer, and this accessor describes what the document holds. A top-level
    /// element outside the subset is skipped, because there is no typed node for it — use
    /// [`MathNode::raw_children`] on [`Math::node`] to see every child element.
    pub fn nodes(&self) -> Vec<MathNode> {
        self.node().children()
    }

    /// The expression this formula contains, or `None` when it has none this layer can describe.
    ///
    /// This is the entry point for reading a formula: it is the first of [`Math::nodes`].
    pub fn expression(&self) -> Option<MathNode> {
        self.node().children().into_iter().next()
    }

    /// Replace the content of this formula with a single expression.
    ///
    /// Every child element of the `math` element is detached (including one this layer cannot
    /// describe) and `expression` is attached in its place. The expression is given the MathML
    /// namespace declaration if the root does not already make it visible, which is what keeps a
    /// formula correct when the root is written with a prefix (see [`MathNode::push`]).
    pub fn set_expression(&self, expression: MathNode) {
        self.node().set_children([expression]);
    }

    /// Append a node as the last child of this formula.
    ///
    /// Prefer [`Math::set_expression`] for a formula with a single expression, which is what the
    /// specification describes; this method is here for the (invalid, but editable) documents that
    /// have more than one, and for a caller that wants to build the content step by step.
    pub fn push(&self, node: MathNode) {
        self.node().push(node);
    }

    /// Detach every child element of this formula.
    pub fn clear(&self) {
        self.node().clear_children();
    }
}

impl XmlDefault for Math {
    fn default(document: Document) -> Self {
        Math::new_empty(document)
    }
}

impl Display for Math {
    /// Renders the whole formula as XML, from the element tree.
    ///
    /// The rendering is the `math` element and its content, exactly as
    /// [`MathNode`] renders a node: what the document holds, with its namespace declarations.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.xml_element().element())
    }
}

#[cfg(test)]
mod tests;
