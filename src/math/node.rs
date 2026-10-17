//! `MathNode`: a typed view of one node of an SBML formula.
//!
//! A formula is a tree of MathML elements. [`MathNode`] is the sum type over the node kinds the
//! SBML subset contains: the sixteen element kinds (the `math` formula root and the fifteen
//! structural constructs, each a declared wrapper type), plus the operator and constant elements,
//! which are values of a closed enumeration rather than declared types.
//!
//! The type is a *view*, exactly like every other wrapper of this crate: a node holds the element it
//! describes, so reading a node never copies the document and editing a node edits the document.
//! [`MathNode::try_from_element`] is the checked conversion; it answers `None` for an element that
//! is not in the MathML namespace or whose name is outside the subset, and that answer is a *type*
//! verdict, not a statement that the crate cannot read the formula: the element stays reachable
//! through the raw [`XmlElement`], which is the escape hatch for a formula that uses MathML SBML does
//! not permit.
//!
//! Navigation mirrors the tree: [`MathNode::children`] gives the recognized child nodes,
//! [`MathNode::raw_children`] gives every child element, and [`MathNode::parent`] walks up.
//! [`Display`] renders the node's element subtree as XML, taken from the element tree itself.
//!
//! The construction helpers build a subtree in the element order of the specification's content
//! models. They create *detached* nodes: the document of a node is the one the caller passes, and
//! the node enters the document when it is attached to a formula (see [`MathNode::push`] and
//! [`Math::set_expression`]) or when the caller attaches the raw
//! element itself.

use crate::constants::namespaces::{to_xml_namespace, NS_MATHML};
use crate::math::{
    Annotation, AnnotationXml, Apply, BVar, Ci, Cn, CnType, Csymbol, Degree, Lambda, Logbase, Math,
    MathConstant, MathOperator, MathSymbol, Otherwise, Piece, Piecewise, Semantics, Sep,
};
use crate::xml::{Document, OptionalXmlProperty, SbmlObject, XmlElement, XmlWrapper};
use std::fmt::{Display, Formatter};

/// The kind of a [`MathNode`], without the element it describes.
///
/// The value is cheap to copy and compare, so it is what a caller matches on when it only needs to
/// know *what* a node is (for a decision, a log message or a test) rather than to use it. It is kept
/// in step with [`MathNode`] by its `From` implementation, so the two cannot drift apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathNodeKind {
    /// The `math` formula root.
    Math,
    /// The `apply` element.
    Apply,
    /// The `bvar` element.
    BVar,
    /// The `ci` element.
    Ci,
    /// The `cn` element.
    Cn,
    /// The `csymbol` element.
    Csymbol,
    /// The `degree` element.
    Degree,
    /// The `lambda` element.
    Lambda,
    /// The `logbase` element.
    Logbase,
    /// The `otherwise` element.
    Otherwise,
    /// The `piece` element.
    Piece,
    /// The `piecewise` element.
    Piecewise,
    /// The `semantics` element.
    Semantics,
    /// The `sep` element.
    Sep,
    /// The `annotation` element.
    Annotation,
    /// The `annotation-xml` element.
    AnnotationXml,
    /// An operator element (`<plus/>`, `<divide/>`, …), which the subset fixes as a closed set.
    Operator(MathOperator),
    /// A constant element (`<pi/>`, `<true/>`, …), which the subset fixes as a closed set.
    Constant(MathConstant),
}

impl MathNodeKind {
    /// `true` when this kind is one of the declared element kinds, i.e. when the node it describes
    /// is wrapped in a type this crate declares.
    pub fn is_declared_element(self) -> bool {
        !matches!(self, MathNodeKind::Operator(_) | MathNodeKind::Constant(_))
    }

    /// The local name of the element this kind describes.
    pub fn element_name(self) -> &'static str {
        match self {
            MathNodeKind::Math => "math",
            MathNodeKind::Apply => "apply",
            MathNodeKind::BVar => "bvar",
            MathNodeKind::Ci => "ci",
            MathNodeKind::Cn => "cn",
            MathNodeKind::Csymbol => "csymbol",
            MathNodeKind::Degree => "degree",
            MathNodeKind::Lambda => "lambda",
            MathNodeKind::Logbase => "logbase",
            MathNodeKind::Otherwise => "otherwise",
            MathNodeKind::Piece => "piece",
            MathNodeKind::Piecewise => "piecewise",
            MathNodeKind::Semantics => "semantics",
            MathNodeKind::Sep => "sep",
            MathNodeKind::Annotation => "annotation",
            MathNodeKind::AnnotationXml => "annotation-xml",
            MathNodeKind::Operator(it) => it.name(),
            MathNodeKind::Constant(it) => it.name(),
        }
    }
}

impl Display for MathNodeKind {
    /// Writes the element name of the kind.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.element_name())
    }
}

/// One node of an SBML formula.
///
/// The variants of this enumeration are the node kinds of the subset; see the module documentation
/// for what the type is and what it is not. [`MathNode::Operator`] and [`MathNode::Constant`] carry
/// their element explicitly, because an operator or constant is identified by its value *and* has a
/// position in the tree (its text, its attributes such as `definitionURL`, its parent).
#[derive(Clone, Debug)]
pub enum MathNode {
    /// The `math` formula root, i.e. the node returned by [`Math::node`].
    Math(Math),
    /// An `apply` element.
    Apply(Apply),
    /// A `bvar` element.
    BVar(BVar),
    /// A `ci` element.
    Ci(Ci),
    /// A `cn` element.
    Cn(Cn),
    /// A `csymbol` element.
    Csymbol(Csymbol),
    /// A `degree` element.
    Degree(Degree),
    /// A `lambda` element.
    Lambda(Lambda),
    /// A `logbase` element.
    Logbase(Logbase),
    /// An `otherwise` element.
    Otherwise(Otherwise),
    /// A `piece` element.
    Piece(Piece),
    /// A `piecewise` element.
    Piecewise(Piecewise),
    /// A `semantics` element.
    Semantics(Semantics),
    /// A `sep` element.
    Sep(Sep),
    /// An `annotation` element.
    Annotation(Annotation),
    /// An `annotation-xml` element.
    AnnotationXml(AnnotationXml),
    /// An operator element of the closed operator set.
    Operator {
        /// The operator this element is.
        operator: MathOperator,
        /// The element itself, which is what navigation and rendering work on.
        element: XmlElement,
    },
    /// A constant element of the closed constant set.
    Constant {
        /// The constant this element is.
        constant: MathConstant,
        /// The element itself, which is what navigation and rendering work on.
        element: XmlElement,
    },
}

impl From<MathNode> for XmlElement {
    /// The element a node describes.
    fn from(value: MathNode) -> Self {
        value.into_element()
    }
}

impl MathNode {
    /// View `element` as a formula node, or answer `None` when it is outside the SBML subset.
    ///
    /// The element has to be in the MathML namespace, and its local name has to be one of the
    /// seventy-three names the subset permits inside a `math` element, or `math` itself. The
    /// dispatch on the name is only a table: whether the element really is the declared type is
    /// decided by that type's own checked conversion
    /// ([`SbmlObject::try_from_element`]), so a node and
    /// its declared identity cannot disagree.
    ///
    /// `None` is a *type* verdict. It never means that the element cannot be read: the argument is
    /// returned unchanged through the caller's own handle, and it is reachable again through
    /// [`XmlElement`] navigation. A `<math>` element in a namespace other than MathML is outside the
    /// subset as well, because rule 10201 makes it a document error rather than a formula.
    pub fn try_from_element(element: XmlElement) -> Option<MathNode> {
        if element.namespace_url() != NS_MATHML.1 {
            return None;
        }
        match element.tag_name().as_str() {
            "math" => Math::try_from_element(element).map(MathNode::Math),
            "apply" => Apply::try_from_element(element).map(MathNode::Apply),
            "bvar" => BVar::try_from_element(element).map(MathNode::BVar),
            "ci" => Ci::try_from_element(element).map(MathNode::Ci),
            "cn" => Cn::try_from_element(element).map(MathNode::Cn),
            "csymbol" => Csymbol::try_from_element(element).map(MathNode::Csymbol),
            "degree" => Degree::try_from_element(element).map(MathNode::Degree),
            "lambda" => Lambda::try_from_element(element).map(MathNode::Lambda),
            "logbase" => Logbase::try_from_element(element).map(MathNode::Logbase),
            "otherwise" => Otherwise::try_from_element(element).map(MathNode::Otherwise),
            "piece" => Piece::try_from_element(element).map(MathNode::Piece),
            "piecewise" => Piecewise::try_from_element(element).map(MathNode::Piecewise),
            "semantics" => Semantics::try_from_element(element).map(MathNode::Semantics),
            "sep" => Sep::try_from_element(element).map(MathNode::Sep),
            "annotation" => Annotation::try_from_element(element).map(MathNode::Annotation),
            "annotation-xml" => {
                AnnotationXml::try_from_element(element).map(MathNode::AnnotationXml)
            }
            _ => {
                if let Some(operator) = MathOperator::from_name(&element.tag_name()) {
                    return Some(MathNode::Operator { operator, element });
                }
                MathConstant::from_name(&element.tag_name())
                    .map(|constant| MathNode::Constant { constant, element })
            }
        }
    }

    /// The element this node describes.
    pub fn xml_element(&self) -> &XmlElement {
        match self {
            MathNode::Math(it) => it.xml_element(),
            MathNode::Apply(it) => it.xml_element(),
            MathNode::BVar(it) => it.xml_element(),
            MathNode::Ci(it) => it.xml_element(),
            MathNode::Cn(it) => it.xml_element(),
            MathNode::Csymbol(it) => it.xml_element(),
            MathNode::Degree(it) => it.xml_element(),
            MathNode::Lambda(it) => it.xml_element(),
            MathNode::Logbase(it) => it.xml_element(),
            MathNode::Otherwise(it) => it.xml_element(),
            MathNode::Piece(it) => it.xml_element(),
            MathNode::Piecewise(it) => it.xml_element(),
            MathNode::Semantics(it) => it.xml_element(),
            MathNode::Sep(it) => it.xml_element(),
            MathNode::Annotation(it) => it.xml_element(),
            MathNode::AnnotationXml(it) => it.xml_element(),
            MathNode::Operator { element, .. } => element,
            MathNode::Constant { element, .. } => element,
        }
    }

    /// The element this node describes, consumed.
    pub fn into_element(self) -> XmlElement {
        match self {
            MathNode::Math(it) => it.into(),
            MathNode::Apply(it) => it.into(),
            MathNode::BVar(it) => it.into(),
            MathNode::Ci(it) => it.into(),
            MathNode::Cn(it) => it.into(),
            MathNode::Csymbol(it) => it.into(),
            MathNode::Degree(it) => it.into(),
            MathNode::Lambda(it) => it.into(),
            MathNode::Logbase(it) => it.into(),
            MathNode::Otherwise(it) => it.into(),
            MathNode::Piece(it) => it.into(),
            MathNode::Piecewise(it) => it.into(),
            MathNode::Semantics(it) => it.into(),
            MathNode::Sep(it) => it.into(),
            MathNode::Annotation(it) => it.into(),
            MathNode::AnnotationXml(it) => it.into(),
            MathNode::Operator { element, .. } => element,
            MathNode::Constant { element, .. } => element,
        }
    }

    /// The kind of this node.
    pub fn kind(&self) -> MathNodeKind {
        match self {
            MathNode::Math(_) => MathNodeKind::Math,
            MathNode::Apply(_) => MathNodeKind::Apply,
            MathNode::BVar(_) => MathNodeKind::BVar,
            MathNode::Ci(_) => MathNodeKind::Ci,
            MathNode::Cn(_) => MathNodeKind::Cn,
            MathNode::Csymbol(_) => MathNodeKind::Csymbol,
            MathNode::Degree(_) => MathNodeKind::Degree,
            MathNode::Lambda(_) => MathNodeKind::Lambda,
            MathNode::Logbase(_) => MathNodeKind::Logbase,
            MathNode::Otherwise(_) => MathNodeKind::Otherwise,
            MathNode::Piece(_) => MathNodeKind::Piece,
            MathNode::Piecewise(_) => MathNodeKind::Piecewise,
            MathNode::Semantics(_) => MathNodeKind::Semantics,
            MathNode::Sep(_) => MathNodeKind::Sep,
            MathNode::Annotation(_) => MathNodeKind::Annotation,
            MathNode::AnnotationXml(_) => MathNodeKind::AnnotationXml,
            MathNode::Operator { operator, .. } => MathNodeKind::Operator(*operator),
            MathNode::Constant { constant, .. } => MathNodeKind::Constant(*constant),
        }
    }

    /// `true` when this element really is the node the wrapper claims.
    ///
    /// A `MathNode` is built by [`MathNode::try_from_element`] or by a construction helper, so a
    /// node is checked when it is created; this method re-checks an element that a caller obtained
    /// elsewhere (a node whose element was edited by hand, for example).
    pub fn is_element(&self) -> bool {
        MathNode::try_from_element(self.xml_element().clone())
            .is_some_and(|it| it.kind() == self.kind())
    }

    /// The child *elements* of this node, as typed nodes.
    ///
    /// A child outside the SBML subset is skipped, because there is no typed node to return for it;
    /// [`MathNode::raw_children`] returns every child element, which is the escape hatch. The order
    /// is document order.
    pub fn children(&self) -> Vec<MathNode> {
        self.xml_element()
            .child_elements()
            .into_iter()
            .filter_map(MathNode::try_from_element)
            .collect()
    }

    /// Every child element of this node, whether or not the subset contains it.
    pub fn raw_children(&self) -> Vec<XmlElement> {
        self.xml_element().child_elements()
    }

    /// The parent of this node as a typed node, or `None` when the parent is outside the subset
    /// (an SBML object, for example: the `math` element of a `kineticLaw` has a parent that is not
    /// a MathML element).
    ///
    /// The raw parent element is [`MathNode::xml_element`] followed by
    /// [`XmlElement::parent`].
    pub fn parent(&self) -> Option<MathNode> {
        self.xml_element()
            .parent()
            .and_then(MathNode::try_from_element)
    }

    /// Declare the MathML namespace on this element when it is not already in scope.
    ///
    /// A subtree built by the construction helpers deliberately carries no namespace declaration of
    /// its own: the formula root declares it, and every node below inherits it. A subtree that is
    /// serialized *without* being attached to a formula therefore needs one declaration, and this is
    /// where it comes from. Calling it on a node inside a formula does nothing.
    pub fn ensure_namespace(&self) {
        let namespace =
            to_xml_namespace(NS_MATHML).expect("The MathML namespace is a valid XML namespace.");
        self.xml_element().ensure_namespace(&namespace);
    }

    /// Append `node` as the last child of this node.
    ///
    /// The child is attached first and given a namespace declaration second, and the declaration is
    /// written only when the node enters a *live* tree (one that is attached to a document): a node
    /// built inside a subtree inherits the declaration of the subtree's root, so building
    /// `apply(plus, [cn, cn])` and attaching the result to a formula produces exactly one MathML
    /// namespace declaration for the whole formula. Attaching a subtree to a node of a document that
    /// does not make the MathML namespace visible gives the child the declaration it needs (a
    /// document that binds MathML under a prefix, for example).
    ///
    /// A node is a *view* of an element, and an element has one parent: attaching a node that is
    /// already a child of another element moves it rather than copying it. A caller that needs the
    /// same subexpression twice builds it twice (each construction helper creates a fresh element).
    pub fn push(&self, node: MathNode) {
        let parent = self.xml_element();
        parent
            .element()
            .append_child(node.xml_element().element().node());
        if parent.element().is_attached() {
            node.ensure_namespace();
        }
    }

    /// Replace the children of this node with `nodes`, in order.
    ///
    /// The namespace is handled as in [`MathNode::push`].
    pub fn set_children(&self, nodes: impl IntoIterator<Item = MathNode>) {
        self.clear_children();
        for node in nodes {
            self.push(node);
        }
    }

    /// Detach every child node of this element, including the ones the subset does not contain.
    pub fn clear_children(&self) {
        for child in self.xml_element().element().children() {
            child.detach();
        }
    }

    /// The local name of the element this node describes.
    pub fn element_name(&self) -> &'static str {
        self.kind().element_name()
    }
}

/// The `From` conversions from a declared wrapper into the sum type.
///
/// A declared element type is one variant of [`MathNode`]; these conversions are what the
/// construction helpers use to put a fresh element into the sum type, and they are public because a
/// caller that already holds a wrapper (from [`MathNode::try_from_element`], for example) should not
/// have to re-match it.
macro_rules! math_node_from_wrapper {
    ($($wrapper:ident => $variant:ident,)*) => {
        $(
            impl From<$wrapper> for MathNode {
                fn from(value: $wrapper) -> Self {
                    MathNode::$variant(value)
                }
            }
        )*
    };
}

math_node_from_wrapper! {
    Math => Math,
    Apply => Apply,
    BVar => BVar,
    Ci => Ci,
    Cn => Cn,
    Csymbol => Csymbol,
    Degree => Degree,
    Lambda => Lambda,
    Logbase => Logbase,
    Otherwise => Otherwise,
    Piece => Piece,
    Piecewise => Piecewise,
    Semantics => Semantics,
    Sep => Sep,
    Annotation => Annotation,
    AnnotationXml => AnnotationXml,
}

/// A detached element of the declared type `T`, carrying the MathML namespace in its name but no
/// namespace declaration of its own.
///
/// The name and the namespace come from the declared identity of `T`, so a fresh element and the
/// elements the type recognizes cannot disagree; what the element deliberately lacks is the
/// declaration, because a formula built at once has to carry exactly one of them, on its root (see
/// the module documentation of [`crate::math`] and [`MathNode::push`]).
fn new_math_element<T: SbmlObject>(document: &Document) -> T {
    T::from(XmlElement::new_qualified_undeclared(
        document.clone(),
        T::ELEMENT_NAME,
        T::ELEMENT_NAMESPACE,
    ))
}

/// A detached element of type `T` with `children` attached, in order.
fn new_container<T: SbmlObject + Into<MathNode>>(
    document: &Document,
    children: impl IntoIterator<Item = MathNode>,
) -> MathNode {
    let node: MathNode = new_math_element::<T>(document).into();
    for child in children {
        node.push(child);
    }
    node
}
/// A detached MathML element with the given local name, without a namespace declaration of its own.
///
/// This is the name-based sibling of [`new_math_element`], needed for the operator and constant
/// elements: they are identified by their name alone and have no declared type to take the name from
/// (see the module documentation of [`crate::math`]). Everything else is created through its
/// declared identity.
fn new_math_name_element(document: &Document, name: &str) -> XmlElement {
    let qualified_name = crate::constants::namespaces::qualified_name(NS_MATHML, name)
        .expect("A MathML element name of the subset is a valid XML name.");
    XmlElement::new(document.create_element(qualified_name))
}

impl MathOperator {
    /// A detached element of this operator, e.g. a `<plus/>`.
    ///
    /// An operator element is an ordinary child of an `apply`, so a caller that builds an
    /// application by hand (or changes the operator of an existing one) needs it;
    /// [`MathNode::apply`] writes it for the caller.
    pub fn node(self, document: Document) -> MathNode {
        MathNode::Operator {
            operator: self,
            element: new_math_name_element(&document, self.name()),
        }
    }
}

impl MathConstant {
    /// A detached element of this constant, e.g. a `<pi/>`.
    pub fn node(self, document: Document) -> MathNode {
        MathNode::Constant {
            constant: self,
            element: new_math_name_element(&document, self.name()),
        }
    }
}

/// The construction helpers of [`MathNode`].
///
/// Every helper takes the document the new elements belong to — a document is a cheap, cloneable
/// handle, so passing it by value costs nothing — and returns a *detached* node. The children are
/// written in the order the specification's content models fix, which is what makes the result a
/// formula a reader recognizes: the operator of an `apply` first, the `degree` of a `root` or the
/// `logbase` of a `log` immediately after the operator, the bound variables of a `lambda` before its
/// body, the value of a `piece` before its condition, and so on.
///
/// Attach the result to a formula with
/// [`Math::set_expression`], which creates the root or replaces
/// the expression of an existing one, or with [`MathNode::push`].
impl MathNode {
    /// An `<apply>` element: the built-in `operator` applied to `arguments`.
    ///
    /// ```rust
    /// use biodivine_lib_sbml::math::*;
    /// use biodivine_lib_sbml::xml::Document;
    ///
    /// let document = Document::parse_string("<?xml version=\"1.0\"?><probe/>")
    ///     .expect("A well-formed document.");
    /// let formula = Math::new_empty(document.clone());
    /// formula.set_expression(MathNode::apply(
    ///     document.clone(),
    ///     MathOperator::Times,
    ///     [
    ///         MathNode::ci(document.clone(), "k"),
    ///         MathNode::integer(document, 2),
    ///     ],
    /// ));
    ///
    /// // The `<math>` root carries the one MathML namespace declaration of the formula.
    /// assert_eq!(formula.expression().unwrap().element_name(), "apply");
    /// assert!(formula.to_string().starts_with(
    ///     "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><apply><times/><ci>k</ci>"
    /// ));
    /// ```
    pub fn apply(
        document: Document,
        operator: MathOperator,
        arguments: impl IntoIterator<Item = MathNode>,
    ) -> MathNode {
        let operator = operator.node(document.clone());
        Self::apply_with(document, operator, arguments)
    }

    /// An `<apply>` element whose operator is *any* node, not necessarily a built-in operator.
    ///
    /// This is the general form, and it is what keeps the whole formula language representable: an
    /// `apply` whose operator is a `csymbol` (the SBML `rateOf` symbol, for example), a `ci` (a call
    /// to a `functionDefinition`), or any other element the subset permits as an operator.
    pub fn apply_with(
        document: Document,
        operator: MathNode,
        arguments: impl IntoIterator<Item = MathNode>,
    ) -> MathNode {
        let node = MathNode::Apply(new_math_element::<Apply>(&document));
        node.push(operator);
        for argument in arguments {
            node.push(argument);
        }
        node
    }

    /// A `<ci>` element that refers to `symbol`, which is written as its text content.
    pub fn ci(document: Document, symbol: &str) -> MathNode {
        let element = new_math_element::<Ci>(&document);
        element.set_symbol(symbol);
        MathNode::Ci(element)
    }

    /// A `<cn>` element with `payload` as its text content and no `type` attribute.
    ///
    /// This is the general number form: the payload is written verbatim, so a caller that needs a
    /// number of a specific type writes the whole payload itself. The typed forms below write the
    /// `type` attribute (including `real`, which is the default of a missing one) and, for a
    /// two-token number, the separator.
    pub fn cn(document: Document, payload: &str) -> MathNode {
        let element = new_math_element::<Cn>(&document);
        element.set_payload(payload);
        MathNode::Cn(element)
    }

    /// A `<cn type="integer">` element.
    pub fn integer(document: Document, value: i64) -> MathNode {
        Self::number_element(&document, CnType::Integer, value.to_string())
    }

    /// A `<cn type="real">` element.
    ///
    /// The value is written with the shortest representation that reads back as the same number,
    /// which is what the `Display` implementation of `f64` produces. A value that is *not* a number
    /// has no `<cn>` spelling in MathML: `NaN` and the infinities are the constant elements
    /// [`MathConstant::NotANumber`] and
    /// [`MathConstant::Infinity`], so a caller that means them
    /// builds those nodes instead of calling this one.
    pub fn real(document: Document, value: f64) -> MathNode {
        Self::number_element(&document, CnType::Real, value.to_string())
    }

    /// A `<cn type="rational">` element: `<cn type="rational">numerator<sep/>denominator</cn>`.
    pub fn rational(document: Document, numerator: i64, denominator: i64) -> MathNode {
        Self::two_token_number(
            document,
            CnType::Rational,
            numerator.to_string(),
            denominator.to_string(),
        )
    }

    /// A `<cn type="e-notation">` element:
    /// `<cn type="e-notation">mantissa<sep/>exponent</cn>`.
    ///
    /// The exponent is the power of ten the mantissa is multiplied by, which is how MathML 2.0
    /// defines `e-notation` (it is not the `1.2e-3` spelling of programming languages). As
    /// [`MathNode::real`] documents, a mantissa that is not a number has no `<cn>` spelling.
    pub fn e_notation(document: Document, mantissa: f64, exponent: i32) -> MathNode {
        Self::two_token_number(
            document,
            CnType::ENotation,
            mantissa.to_string(),
            exponent.to_string(),
        )
    }

    /// A `<sep/>` element, the separator of a two-token number.
    pub fn sep(document: Document) -> MathNode {
        MathNode::Sep(new_math_element::<Sep>(&document))
    }

    /// A `<csymbol>` element that denotes one of the four SBML symbols.
    ///
    /// The element is written the way the MathML schema of the specification describes it:
    /// `<csymbol definitionURL="…" encoding="text">name</csymbol>`.
    pub fn csymbol(document: Document, symbol: MathSymbol, name: &str) -> MathNode {
        Self::csymbol_with_url(document, symbol.definition_url(), name)
    }

    /// A `<csymbol>` element that denotes a symbol of another namespace, identified by `url`.
    ///
    /// Rule 10205 permits a package to define symbols of its own, so the URL is not restricted to
    /// the four SBML symbols here; a `csymbol` whose URL is outside the permitted set is reported by
    /// the validation layer.
    pub fn csymbol_with_url(document: Document, url: &str, name: &str) -> MathNode {
        let element = new_math_element::<Csymbol>(&document);
        element.definition_url().set_some(&url.to_string());
        element.encoding().set_some(&"text".to_string());
        element.set_symbol(name);
        MathNode::Csymbol(element)
    }

    /// A `<bvar>` element that declares `name` as a bound variable.
    pub fn bvar(document: Document, name: &str) -> MathNode {
        let node = MathNode::BVar(new_math_element::<BVar>(&document));
        node.push(Self::ci(document, name));
        node
    }

    /// A `<lambda>` element: a function with `arguments` as its bound variables and `body` as the
    /// expression that computes its value.
    ///
    /// A `lambda` is permitted only inside the `math` element of a `functionDefinition` (rule
    /// 10208); the builder writes it wherever the caller attaches it.
    pub fn lambda(document: Document, arguments: &[&str], body: MathNode) -> MathNode {
        let node = MathNode::Lambda(new_math_element::<Lambda>(&document));
        for argument in arguments {
            node.push(Self::bvar(document.clone(), argument));
        }
        node.push(body);
        node
    }

    /// A `<degree>` element: the degree of a `root`.
    pub fn degree(document: Document, value: MathNode) -> MathNode {
        new_container::<Degree>(&document, [value])
    }

    /// A `<logbase>` element: the base of a `log`.
    pub fn logbase(document: Document, value: MathNode) -> MathNode {
        new_container::<Logbase>(&document, [value])
    }

    /// An `<apply>` that takes the `root` of `radicand`, optionally of a given `degree`.
    ///
    /// The `degree` is written as the first argument of the application, which is where the content
    /// model puts it and what rule 10218 reads as the second argument of a two-argument `root`.
    pub fn root(document: Document, radicand: MathNode, degree: Option<MathNode>) -> MathNode {
        let operator = MathOperator::Root.node(document.clone());
        let arguments = degree
            .map(|degree| Self::degree(document.clone(), degree))
            .into_iter()
            .chain([radicand]);
        Self::apply_with(document, operator, arguments)
    }

    /// An `<apply>` of the `log` operator, optionally to a given `base`.
    ///
    /// The `base` is written as a `logbase` element immediately after the operator, which is the
    /// order the content model fixes.
    pub fn log(document: Document, argument: MathNode, base: Option<MathNode>) -> MathNode {
        let operator = MathOperator::Log.node(document.clone());
        let arguments = base
            .map(|base| Self::logbase(document.clone(), base))
            .into_iter()
            .chain([argument]);
        Self::apply_with(document, operator, arguments)
    }

    /// A `<piece>` element: the `value` that applies when `condition` holds.
    pub fn piece(document: Document, value: MathNode, condition: MathNode) -> MathNode {
        new_container::<Piece>(&document, [value, condition])
    }

    /// An `<otherwise>` element: the value of the default case of a `piecewise`.
    pub fn otherwise(document: Document, value: MathNode) -> MathNode {
        new_container::<Otherwise>(&document, [value])
    }

    /// A `<piecewise>` element: the `(value, condition)` cases first, then the optional otherwise
    /// case, in that order.
    ///
    /// A `piecewise` with neither a `piece` nor an `otherwise` is reported by rule 10218; the
    /// builder writes what it is asked for.
    pub fn piecewise(
        document: Document,
        pieces: impl IntoIterator<Item = (MathNode, MathNode)>,
        otherwise: Option<MathNode>,
    ) -> MathNode {
        let cases = pieces
            .into_iter()
            .map(|(value, condition)| Self::piece(document.clone(), value, condition));
        let node = new_container::<Piecewise>(&document, cases);
        if let Some(otherwise) = otherwise {
            node.push(Self::otherwise(document, otherwise));
        }
        node
    }

    /// A `<semantics>` element: the annotated `expression`, then `annotations` in order.
    pub fn semantics(
        document: Document,
        expression: MathNode,
        annotations: impl IntoIterator<Item = MathNode>,
    ) -> MathNode {
        let node = new_container::<Semantics>(&document, [expression]);
        for annotation in annotations {
            node.push(annotation);
        }
        node
    }

    /// An `<annotation>` element with the given `encoding` and text content.
    pub fn annotation(document: Document, encoding: &str, text: &str) -> MathNode {
        let element = new_math_element::<Annotation>(&document);
        element.encoding().set_some(&encoding.to_string());
        element.set_payload(text);
        MathNode::Annotation(element)
    }

    /// An empty `<annotation-xml>` element with the given `encoding`.
    ///
    /// The content of an `annotation-xml` is arbitrary markup — the same expression in another
    /// encoding, an RDF description, … — so it is the caller who attaches it, through
    /// [`MathNode::push`] for a formula node or through the raw element for anything else.
    pub fn annotation_xml(document: Document, encoding: &str) -> MathNode {
        let element = new_math_element::<AnnotationXml>(&document);
        element.encoding().set_some(&encoding.to_string());
        MathNode::AnnotationXml(element)
    }

    /// A one-token number element of the given type.
    fn number_element(document: &Document, number_type: CnType, payload: String) -> MathNode {
        let element = new_math_element::<Cn>(document);
        element.type_().set_some(&number_type);
        element.set_payload(&payload);
        MathNode::Cn(element)
    }

    /// A two-token number element of the given type: the first token, a `sep`, the second token.
    ///
    /// The two tokens are text nodes, not elements, so the second one is appended to the element
    /// directly; the `sep` is the only *element* the content model of a `cn` contains.
    fn two_token_number(
        document: Document,
        number_type: CnType,
        first: String,
        second: String,
    ) -> MathNode {
        let node = Self::number_element(&document, number_type, first);
        node.push(Self::sep(document.clone()));
        let text = document
            .create_text(second)
            .expect("A number is valid XML text.");
        node.xml_element().element().append_child(text);
        node
    }
}

impl Display for MathNode {
    /// Renders the element subtree of this node as XML.
    ///
    /// The rendering comes from the element tree through the document serializer, so the
    /// declarations the tree holds are the ones that are written (a subtree that has not entered a
    /// document declares nothing until [`MathNode::ensure_namespace`] is called), text and attribute
    /// values are escaped as XML requires, and nothing is invented: what is printed is what the
    /// document holds. This is the *XML form* of the node, not an infix rendering of the formula.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.xml_element().element())
    }
}
