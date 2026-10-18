//! The typed MathML layer: formulas as typed nodes.
//!
//! An SBML formula is a tree of MathML elements, and the library models it with two types:
//! [`Math`](biodivine_lib_sbml::math::Math) (the `<math>` element that holds a formula) and
//! [`MathNode`](biodivine_lib_sbml::math::MathNode) (one node of the tree, the sum type over the
//! element kinds the SBML subset contains). This module binds both, plus the closed operator,
//! constant and symbol sets.
//!
//! The Python shape follows the same idea as the rest of the bindings: a node is a *view* of one
//! element, so reading a node reads the document and building a node creates detached elements that
//! the caller attaches. The one place where Python and Rust differ is the *kind* of a node: in Rust
//! a caller matches on the `MathNode` enumeration, while Python gets a `MathNodeKind` value with the
//! element name, and the operator and constant classes carry the string spellings the document uses.
//! The kind is a value rather than a class per element because a Python user needs to compare and
//! print it, not to pattern-match on it.

use biodivine_lib_sbml::math::{
    Math, MathArity, MathConstant, MathNode, MathNodeKind, MathOperator, MathSymbol, SbmlUnits,
};
use biodivine_lib_sbml::xml::{SbmlObject, XmlElement, XmlProperty, XmlPropertyType, XmlWrapper};
use biodivine_lib_xml_dom_sys::PyDocument;
use pyo3::prelude::*;

use crate::element::PyElement;
use crate::error::value_err;

/// The `<math>` element: the formula of a `kineticLaw`, a rule, a constraint, an event assignment
/// and every other formula site of the specification.
#[pyclass(name = "Math", module = "biodivine_sys.sbml", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyMath {
    pub(crate) inner: Math,
}

impl From<Math> for PyMath {
    fn from(inner: Math) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyMath {
    /// Creates a detached `<math>` element holding `expression`.
    #[staticmethod]
    fn new(document: &PyDocument, expression: &PyMathNode) -> PyMath {
        PyMath::from(Math::new(
            document.rust_document().clone(),
            expression.inner.clone(),
        ))
    }

    /// Creates an empty `<math>` element of `document`.
    #[staticmethod]
    fn new_empty(document: &PyDocument) -> PyMath {
        PyMath::from(Math::new_empty(document.rust_document().clone()))
    }

    /// Creates an empty `<math>` element of `document` (the `XmlDefault` constructor).
    #[staticmethod]
    fn default(document: &PyDocument) -> PyMath {
        PyMath::from(<Math as biodivine_lib_sbml::xml::XmlDefault>::default(
            document.rust_document().clone(),
        ))
    }

    /// The formula of this element as a typed node.
    ///
    /// Raises:
    ///     ValueError: if the element does not hold a formula of the SBML subset.
    #[getter]
    fn node(&self) -> PyResult<PyMathNode> {
        self.inner
            .expression()
            .map(PyMathNode::from)
            .ok_or_else(|| value_err("the <math> element does not hold a formula".to_string()))
    }

    /// Every child node of the formula, in document order.
    #[getter]
    fn nodes(&self) -> Vec<PyMathNode> {
        self.inner
            .nodes()
            .into_iter()
            .map(PyMathNode::from)
            .collect()
    }

    /// The formula of this element, or `None` when it is empty.
    #[getter]
    fn expression(&self) -> Option<PyMathNode> {
        self.inner.expression().map(PyMathNode::from)
    }

    /// Replaces the formula of this element.
    fn set_expression(&self, expression: &PyMathNode) {
        self.inner.set_expression(expression.inner.clone());
    }

    /// Appends `node` as a child of this element.
    fn push(&self, node: &PyMathNode) {
        self.inner.push(node.inner.clone());
    }

    /// Removes every child of this element.
    fn clear(&self) {
        self.inner.clear();
    }

    /// The XML element of this object.
    #[getter]
    fn element(&self) -> PyElement {
        PyElement::from(XmlWrapper::xml_element(&self.inner).element().clone())
    }

    /// The local name of the element this type recognizes.
    #[classattr]
    const ELEMENT_NAME: &'static str = <Math as SbmlObject>::ELEMENT_NAME;

    /// The namespace of the element this type recognizes, as a `(prefix, url)` pair.
    #[classattr]
    const ELEMENT_NAMESPACE: (&'static str, &'static str) = <Math as SbmlObject>::ELEMENT_NAMESPACE;

    /// The `xsi:type` value that tells this type apart from a sibling, or `None`.
    #[classattr]
    const XSI_TYPE: Option<&'static str> = <Math as SbmlObject>::XSI_TYPE;

    /// Every local name this type recognizes.
    #[staticmethod]
    fn element_names() -> Vec<&'static str> {
        <Math as SbmlObject>::element_names().to_vec()
    }

    /// Whether `element` is a `<math>` element of the MathML namespace.
    #[staticmethod]
    fn matches_element(element: &PyElement) -> bool {
        <Math as SbmlObject>::matches_element(&XmlElement::new(element.rust_element().clone()))
    }

    /// Whether this object is a valid `<math>` element.
    fn is_element(&self) -> bool {
        SbmlObject::is_element(&self.inner)
    }

    /// This type's view of `element`, or `None` when it is not a `<math>` element.
    #[staticmethod]
    fn try_from_element(element: &PyElement) -> Option<PyMath> {
        <Math as SbmlObject>::try_from_element(XmlElement::new(element.rust_element().clone()))
            .map(PyMath::from)
    }

    fn __repr__(&self) -> String {
        match self.inner.expression() {
            Some(node) => format!("Math({})", node.element_name()),
            None => "Math(empty)".to_string(),
        }
    }
}

/// One node of an SBML formula.
#[pyclass(name = "MathNode", module = "biodivine_sys.sbml", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyMathNode {
    pub(crate) inner: MathNode,
}

impl From<MathNode> for PyMathNode {
    fn from(inner: MathNode) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyMathNode {
    /// This node's element, or `None` when the node is not in the SBML MathML subset.
    #[staticmethod]
    fn try_from_element(element: &PyElement) -> Option<PyMathNode> {
        MathNode::try_from_element(XmlElement::new(element.rust_element().clone()))
            .map(PyMathNode::from)
    }

    /// The XML element of this node.
    #[getter]
    fn element(&self) -> PyElement {
        PyElement::from(self.inner.xml_element().element().clone())
    }

    /// The XML element of this node, as a new handle.
    #[pyo3(name = "into_element")]
    fn element_copy(&self) -> PyElement {
        PyElement::from(self.inner.clone().into_element().element().clone())
    }

    /// What this node is: the element name, as a `MathNodeKind`.
    #[getter]
    fn kind(&self) -> PyMathNodeKind {
        PyMathNodeKind::from(self.inner.kind())
    }

    /// Whether this node is an element the library declares as a type (as opposed to an operator or
    /// a constant, which the subset fixes as closed sets of names).
    fn is_element(&self) -> bool {
        self.inner.is_element()
    }

    /// The child nodes of this node that are in the subset, in document order.
    #[getter]
    fn children(&self) -> Vec<PyMathNode> {
        self.inner
            .children()
            .into_iter()
            .map(PyMathNode::from)
            .collect()
    }

    /// Every child element of this node, in document order.
    #[getter]
    fn raw_children(&self) -> Vec<PyElement> {
        self.inner
            .raw_children()
            .into_iter()
            .map(|it| PyElement::from(it.element().clone()))
            .collect()
    }

    /// The parent node of this node, or `None` when it has none.
    #[getter]
    fn parent(&self) -> Option<PyMathNode> {
        self.inner.parent().map(PyMathNode::from)
    }

    /// Declares the MathML namespace on this node's element if it is not in scope yet.
    fn ensure_namespace(&self) {
        self.inner.ensure_namespace();
    }

    /// Appends `node` as the last child of this node.
    fn push(&self, node: &PyMathNode) {
        self.inner.push(node.inner.clone());
    }

    /// Replaces every child of this node with `nodes`.
    fn set_children(&self, nodes: Vec<PyMathNode>) {
        self.inner
            .set_children(nodes.into_iter().map(|it| it.inner.clone()));
    }

    /// Removes every child of this node.
    fn clear_children(&self) {
        self.inner.clear_children();
    }

    /// The local name of the element this node is.
    #[getter]
    fn element_name(&self) -> &'static str {
        self.inner.element_name()
    }

    /// A copy of this node (the copy is a handle to the same element).
    fn clone(&self) -> PyMathNode {
        PyMathNode::from(self.inner.clone())
    }

    /// An `<apply>` element: `operator` applied to `arguments`.
    ///
    /// `operator` is either a built-in operator of the subset (`MathOperator.PLUS`, ...) or any
    /// node, which is what the general `apply_with` form accepts.
    ///
    /// Raises:
    ///     TypeError: if `operator` is neither a `MathOperator` nor a `MathNode`.
    #[staticmethod]
    #[pyo3(signature = (document, operator, arguments))]
    fn apply(
        document: &PyDocument,
        operator: &Bound<'_, PyAny>,
        arguments: Vec<PyMathNode>,
    ) -> PyResult<PyMathNode> {
        let document = document.rust_document().clone();
        let arguments = arguments.into_iter().map(|it| it.inner.clone());
        if let Ok(builtin) = operator.extract::<PyMathOperator>() {
            return Ok(PyMathNode::from(MathNode::apply(
                document,
                builtin.inner,
                arguments,
            )));
        }
        if let Ok(node) = operator.extract::<PyMathNode>() {
            return Ok(PyMathNode::from(MathNode::apply_with(
                document,
                node.inner.clone(),
                arguments,
            )));
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "expected a MathOperator or a MathNode as the operator of an apply",
        ))
    }

    /// An `<apply>` element whose operator is any node (a `csymbol`, a `ci`, ...).
    #[staticmethod]
    #[pyo3(signature = (document, operator, arguments))]
    fn apply_with(
        document: &PyDocument,
        operator: &PyMathNode,
        arguments: Vec<PyMathNode>,
    ) -> PyMathNode {
        PyMathNode::from(MathNode::apply_with(
            document.rust_document().clone(),
            operator.inner.clone(),
            arguments.into_iter().map(|it| it.inner.clone()),
        ))
    }

    /// A `<ci>` element: a reference to an identifier of the model.
    #[staticmethod]
    fn ci(document: &PyDocument, symbol: &str) -> PyMathNode {
        PyMathNode::from(MathNode::ci(document.rust_document().clone(), symbol))
    }

    /// A `<cn>` element with the given payload (the spelling the document uses).
    #[staticmethod]
    fn cn(document: &PyDocument, payload: &str) -> PyMathNode {
        PyMathNode::from(MathNode::cn(document.rust_document().clone(), payload))
    }

    /// A `<cn>` element holding an integer.
    #[staticmethod]
    fn integer(document: &PyDocument, value: i64) -> PyMathNode {
        PyMathNode::from(MathNode::integer(document.rust_document().clone(), value))
    }

    /// A `<cn>` element holding a real number.
    #[staticmethod]
    fn real(document: &PyDocument, value: f64) -> PyMathNode {
        PyMathNode::from(MathNode::real(document.rust_document().clone(), value))
    }

    /// A `<cn>` element holding a rational number.
    #[staticmethod]
    fn rational(document: &PyDocument, numerator: i64, denominator: i64) -> PyMathNode {
        PyMathNode::from(MathNode::rational(
            document.rust_document().clone(),
            numerator,
            denominator,
        ))
    }

    /// A `<cn>` element holding a number in e-notation.
    #[staticmethod]
    fn e_notation(document: &PyDocument, mantissa: f64, exponent: i32) -> PyMathNode {
        PyMathNode::from(MathNode::e_notation(
            document.rust_document().clone(),
            mantissa,
            exponent,
        ))
    }

    /// A `<sep/>` element.
    #[staticmethod]
    fn sep(document: &PyDocument) -> PyMathNode {
        PyMathNode::from(MathNode::sep(document.rust_document().clone()))
    }

    /// A `<csymbol>` element for one of the SBML symbols (the `time` and `avogadro` symbols).
    #[staticmethod]
    fn csymbol(document: &PyDocument, symbol: PyMathSymbol, name: &str) -> PyMathNode {
        PyMathNode::from(MathNode::csymbol(
            document.rust_document().clone(),
            symbol.inner,
            name,
        ))
    }

    /// A `<csymbol>` element whose definition URL is given as a string.
    #[staticmethod]
    fn csymbol_with_url(document: &PyDocument, url: &str, name: &str) -> PyMathNode {
        PyMathNode::from(MathNode::csymbol_with_url(
            document.rust_document().clone(),
            url,
            name,
        ))
    }

    /// A `<bvar>` element bound to `name`.
    #[staticmethod]
    fn bvar(document: &PyDocument, name: &str) -> PyMathNode {
        PyMathNode::from(MathNode::bvar(document.rust_document().clone(), name))
    }

    /// A `<lambda>` element over `arguments` with `body`.
    #[staticmethod]
    fn lambda(document: &PyDocument, arguments: Vec<String>, body: &PyMathNode) -> PyMathNode {
        let arguments: Vec<&str> = arguments.iter().map(|it| it.as_str()).collect();
        PyMathNode::from(MathNode::lambda(
            document.rust_document().clone(),
            &arguments,
            body.inner.clone(),
        ))
    }

    /// A `<degree>` element.
    #[staticmethod]
    fn degree(document: &PyDocument, value: &PyMathNode) -> PyMathNode {
        PyMathNode::from(MathNode::degree(
            document.rust_document().clone(),
            value.inner.clone(),
        ))
    }

    /// A `<logbase>` element.
    #[staticmethod]
    fn logbase(document: &PyDocument, value: &PyMathNode) -> PyMathNode {
        PyMathNode::from(MathNode::logbase(
            document.rust_document().clone(),
            value.inner.clone(),
        ))
    }

    /// A `<root>` application of `radicand` with an optional `degree`.
    #[staticmethod]
    #[pyo3(signature = (document, radicand, degree = None))]
    fn root(
        document: &PyDocument,
        radicand: &PyMathNode,
        degree: Option<PyMathNode>,
    ) -> PyMathNode {
        PyMathNode::from(MathNode::root(
            document.rust_document().clone(),
            radicand.inner.clone(),
            degree.map(|it| it.inner),
        ))
    }

    /// A `<log>` application of `argument` with an optional `base`.
    #[staticmethod]
    #[pyo3(signature = (document, argument, base = None))]
    fn log(document: &PyDocument, argument: &PyMathNode, base: Option<PyMathNode>) -> PyMathNode {
        PyMathNode::from(MathNode::log(
            document.rust_document().clone(),
            argument.inner.clone(),
            base.map(|it| it.inner),
        ))
    }

    /// A `<piece>` element: `value` when `condition` holds.
    #[staticmethod]
    fn piece(document: &PyDocument, value: &PyMathNode, condition: &PyMathNode) -> PyMathNode {
        PyMathNode::from(MathNode::piece(
            document.rust_document().clone(),
            value.inner.clone(),
            condition.inner.clone(),
        ))
    }

    /// An `<otherwise>` element.
    #[staticmethod]
    fn otherwise(document: &PyDocument, value: &PyMathNode) -> PyMathNode {
        PyMathNode::from(MathNode::otherwise(
            document.rust_document().clone(),
            value.inner.clone(),
        ))
    }

    /// A `<piecewise>` element over `pieces` with `otherwise`.
    ///
    /// Each piece is a `(value, condition)` pair: the value the formula takes when the condition
    /// holds.
    #[staticmethod]
    #[pyo3(signature = (document, pieces, otherwise = None))]
    fn piecewise(
        document: &PyDocument,
        pieces: Vec<(PyMathNode, PyMathNode)>,
        otherwise: Option<PyMathNode>,
    ) -> PyMathNode {
        PyMathNode::from(MathNode::piecewise(
            document.rust_document().clone(),
            pieces
                .into_iter()
                .map(|(value, condition)| (value.inner, condition.inner)),
            otherwise.map(|it| it.inner),
        ))
    }

    /// A `<semantics>` element over `body` with the given annotations.
    #[staticmethod]
    #[pyo3(signature = (document, body, annotations))]
    fn semantics(
        document: &PyDocument,
        body: &PyMathNode,
        annotations: Vec<PyMathNode>,
    ) -> PyMathNode {
        PyMathNode::from(MathNode::semantics(
            document.rust_document().clone(),
            body.inner.clone(),
            annotations.into_iter().map(|it| it.inner),
        ))
    }

    /// An `<annotation>` element with the given encoding and text.
    #[staticmethod]
    fn annotation(document: &PyDocument, encoding: &str, text: &str) -> PyMathNode {
        PyMathNode::from(MathNode::annotation(
            document.rust_document().clone(),
            encoding,
            text,
        ))
    }

    /// An `<annotation-xml>` element with the given encoding.
    #[staticmethod]
    fn annotation_xml(document: &PyDocument, encoding: &str) -> PyMathNode {
        PyMathNode::from(MathNode::annotation_xml(
            document.rust_document().clone(),
            encoding,
        ))
    }

    /// The element of this node as XML text.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("MathNode({})", self.inner.element_name())
    }

    fn __eq__(&self, other: &PyMathNode) -> bool {
        self.inner.xml_element().element() == other.inner.xml_element().element()
    }
}

/// What a node is: the element name of the node, and whether the library declares that element as a
/// type of its own.
///
/// The class is also the namespace of the names: every element kind of the subset is an attribute of
/// it (`MathNodeKind.CI`, `MathNodeKind.PLUS`, ...), so a Python caller compares
/// `node.kind == MathNodeKind.PI` instead of a string literal.
#[pyclass(
    name = "MathNodeKind",
    module = "biodivine_sys.sbml",
    eq,
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyMathNodeKind {
    /// The local name of the element.
    element_name: &'static str,
    /// Whether the kind is a declared element kind (a declared wrapper type).
    is_declared: bool,
}

impl From<MathNodeKind> for PyMathNodeKind {
    fn from(kind: MathNodeKind) -> Self {
        Self {
            element_name: kind.element_name(),
            is_declared: kind.is_declared_element(),
        }
    }
}

impl PyMathNodeKind {
    /// A kind that has no declared element type of its own (an operator or a constant element).
    pub(crate) fn generic(element_name: &'static str, is_declared: bool) -> Self {
        Self {
            element_name,
            is_declared,
        }
    }
}

#[pymethods]
impl PyMathNodeKind {
    /// The local name of the element this kind describes.
    #[getter]
    fn element_name(&self) -> &'static str {
        self.element_name
    }

    /// Whether this kind is one of the declared element kinds (the wrapper types of the library).
    #[getter]
    fn is_declared_element(&self) -> bool {
        self.is_declared
    }

    /// Whether the element named `name` is one of the declared element kinds.
    #[staticmethod]
    fn is_declared_name(name: &str) -> bool {
        DECLARED_KINDS.contains(&name)
    }

    fn __str__(&self) -> &'static str {
        self.element_name
    }

    fn __repr__(&self) -> String {
        format!("MathNodeKind({})", self.element_name)
    }
}

/// One of the operators of the MathML subset SBML allows in an `<apply>`.
///
/// The class is the namespace of the operator names (`MathOperator.PLUS` is `"plus"`), and the
/// static methods answer the questions the Rust enumeration answers: the name, the operator of a
/// name, and the arity.
#[pyclass(
    name = "MathOperator",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy)]
pub struct PyMathOperator {
    pub(crate) inner: MathOperator,
}

#[pymethods]
impl PyMathOperator {
    /// The element name of this operator.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// The arity the specification gives this operator.
    #[getter]
    fn arity(&self) -> &'static str {
        match self.inner.arity() {
            MathArity::Unary => "unary",
            MathArity::Binary => "binary",
            MathArity::UnaryOrBinary => "unary_or_binary",
            MathArity::Nary => "nary",
        }
    }

    /// Every operator of the subset, as a list of `MathOperator` values.
    #[classattr]
    #[pyo3(name = "ALL")]
    fn all() -> Vec<PyMathOperator> {
        MathOperator::ALL
            .iter()
            .map(|operator| PyMathOperator { inner: *operator })
            .collect()
    }

    /// The operator with the element name `name`, or `None` when the subset has no such operator.
    #[staticmethod]
    fn from_name(name: &str) -> Option<PyMathOperator> {
        MathOperator::from_name(name).map(|inner| PyMathOperator { inner })
    }

    /// A detached element of this operator.
    fn node(&self, document: &PyDocument) -> PyMathNode {
        PyMathNode::from(self.inner.node(document.rust_document().clone()))
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("MathOperator({})", self.inner.name())
    }

    fn __eq__(&self, other: &PyMathOperator) -> bool {
        self.inner == other.inner
    }
}

/// One of the constant elements of the MathML subset SBML allows (`<pi/>`, `<true/>`, ...).
#[pyclass(
    name = "MathConstant",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy)]
pub struct PyMathConstant {
    pub(crate) inner: MathConstant,
}

#[pymethods]
impl PyMathConstant {
    /// The element name of this constant.
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// Every constant of the subset, as a list of `MathConstant` values.
    #[classattr]
    #[pyo3(name = "ALL")]
    fn all() -> Vec<PyMathConstant> {
        MathConstant::ALL
            .iter()
            .map(|constant| PyMathConstant { inner: *constant })
            .collect()
    }

    /// The constant with the element name `name`, or `None` when the subset has no such constant.
    #[staticmethod]
    fn from_name(name: &str) -> Option<PyMathConstant> {
        MathConstant::from_name(name).map(|inner| PyMathConstant { inner })
    }

    /// A detached element of this constant.
    fn node(&self, document: &PyDocument) -> PyMathNode {
        PyMathNode::from(self.inner.node(document.rust_document().clone()))
    }

    fn __str__(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("MathConstant({})", self.inner.name())
    }

    fn __eq__(&self, other: &PyMathConstant) -> bool {
        self.inner == other.inner
    }
}

/// One of the SBML symbols a `<csymbol>` element can refer to.
#[pyclass(
    name = "MathSymbol",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy)]
pub struct PyMathSymbol {
    pub(crate) inner: MathSymbol,
}

#[pymethods]
impl PyMathSymbol {
    /// The definition URL of this symbol.
    #[getter]
    fn definition_url(&self) -> &'static str {
        self.inner.definition_url()
    }

    /// Every symbol of the subset, as a list of `MathSymbol` values.
    #[classattr]
    #[pyo3(name = "ALL")]
    fn all() -> Vec<PyMathSymbol> {
        MathSymbol::ALL
            .iter()
            .map(|symbol| PyMathSymbol { inner: *symbol })
            .collect()
    }

    /// The symbol with the definition URL `url`, or `None` when the subset has no such symbol.
    #[staticmethod]
    fn from_url(url: &str) -> Option<PyMathSymbol> {
        MathSymbol::from_url(url).map(|inner| PyMathSymbol { inner })
    }

    fn __repr__(&self) -> String {
        format!("MathSymbol({})", self.inner.definition_url())
    }

    fn __eq__(&self, other: &PyMathSymbol) -> bool {
        self.inner == other.inner
    }
}

/// The arity of an operator.
#[pyclass(
    name = "MathArity",
    module = "biodivine_sys.sbml",
    eq,
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyMathArity {
    pub(crate) name: &'static str,
}

#[pymethods]
impl PyMathArity {
    /// An operator that takes exactly one argument.
    #[classattr]
    #[pyo3(name = "UNARY")]
    fn unary() -> PyMathArity {
        PyMathArity { name: "unary" }
    }

    /// An operator that takes exactly two arguments.
    #[classattr]
    #[pyo3(name = "BINARY")]
    fn binary() -> PyMathArity {
        PyMathArity { name: "binary" }
    }

    /// An operator that takes one or two arguments.
    #[classattr]
    #[pyo3(name = "UNARY_OR_BINARY")]
    fn unary_or_binary() -> PyMathArity {
        PyMathArity {
            name: "unary_or_binary",
        }
    }

    /// An operator that takes any number of arguments, including none.
    #[classattr]
    #[pyo3(name = "NARY")]
    fn nary() -> PyMathArity {
        PyMathArity { name: "nary" }
    }

    fn __str__(&self) -> &'static str {
        self.name
    }

    fn __repr__(&self) -> String {
        format!("MathArity({})", self.name)
    }
}

/// The unit conversion helpers of an object that carries a `units` attribute.
#[pyclass(name = "SbmlUnits", module = "biodivine_sys.sbml", from_py_object)]
#[derive(Debug, Clone)]
pub struct PySbmlUnits {
    /// The element the `units` attribute lives on. The Rust view borrows an element, and a borrowed
    /// value cannot be stored in a Python class, so the class keeps the element itself and builds
    /// the view per call.
    element: XmlElement,
}

impl From<SbmlUnits<'_>> for PySbmlUnits {
    fn from(view: SbmlUnits<'_>) -> Self {
        Self::from_view(view)
    }
}

impl PySbmlUnits {
    /// Builds the wrapper from the borrowed view by keeping the element it reads.
    pub(crate) fn from_view(view: SbmlUnits<'_>) -> Self {
        Self {
            element: view.element().clone(),
        }
    }

    /// The Rust view of this object's `units` attribute.
    fn units_view(&self) -> SbmlUnits<'_> {
        SbmlUnits::new(&self.element)
    }
}

#[pymethods]
impl PySbmlUnits {
    /// Creates the view of the `units` attribute of `element`.
    #[staticmethod]
    #[pyo3(name = "new")]
    fn create(element: &PyElement) -> PySbmlUnits {
        let element = XmlElement::new(element.rust_element().clone());
        PySbmlUnits { element }
    }

    /// The value of the `units` attribute, or `None` when it is not set.
    fn get(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&self.units_view()).map_err(value_err)
    }

    /// Whether the `units` attribute is set.
    fn is_set(&self) -> bool {
        XmlProperty::is_set(&self.units_view())
    }

    /// The raw text of the `units` attribute, or `None` when it is not set.
    fn get_raw(&self) -> Option<String> {
        XmlProperty::get_raw(&self.units_view())
    }

    /// The value of the `units` attribute, or a message when it cannot be read.
    fn get_checked(&self) -> PyResult<Option<String>> {
        XmlProperty::get_checked(&self.units_view()).map_err(value_err)
    }

    /// Sets the `units` attribute from a value (the `OptionalXmlProperty` form).
    fn set_some(&self, value: &str) -> PyResult<()> {
        let view = self.units_view();
        match XmlPropertyType::set(&value.to_string()) {
            Some(text) => XmlProperty::set_raw(&view, text).map_err(value_err),
            None => XmlProperty::clear(&view).map_err(value_err),
        }
    }

    /// Sets the `units` attribute.
    fn set(&self, value: &str) {
        let view = self.units_view();
        let text = value.to_string();
        match XmlPropertyType::set(&text) {
            Some(text) => {
                let _ = XmlProperty::set_raw(&view, text);
            }
            None => {
                let _ = XmlProperty::clear(&view);
            }
        }
    }

    /// Sets the `units` attribute from raw text.
    fn set_raw(&self, value: &str) -> PyResult<()> {
        XmlProperty::set_raw(&self.units_view(), value.to_string()).map_err(value_err)
    }

    /// Removes the `units` attribute.
    fn clear(&self) -> PyResult<()> {
        XmlProperty::clear(&self.units_view()).map_err(value_err)
    }

    /// The element the view reads.
    #[getter]
    fn element(&self) -> PyElement {
        PyElement::from(XmlProperty::element(&self.units_view()).element().clone())
    }

    fn __repr__(&self) -> String {
        match XmlProperty::get_raw(&self.units_view()) {
            Some(value) => format!("SbmlUnits({value})"),
            None => "SbmlUnits(unset)".to_string(),
        }
    }
}

/// Adds the MathML classes and their element-name constants to `module`.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyMath>()?;
    module.add_class::<PyMathNode>()?;
    module.add_class::<PyMathNodeKind>()?;
    module.add_class::<PyMathOperator>()?;
    module.add_class::<PyMathConstant>()?;
    module.add_class::<PyMathSymbol>()?;
    module.add_class::<PyMathArity>()?;
    module.add_class::<PySbmlUnits>()?;

    Ok(())
}

/// The local names of the declared element kinds of the subset (the wrapper types of the library).
const DECLARED_KINDS: [&str; 16] = [
    "math",
    "apply",
    "bvar",
    "ci",
    "cn",
    "csymbol",
    "degree",
    "lambda",
    "logbase",
    "otherwise",
    "piece",
    "piecewise",
    "semantics",
    "sep",
    "annotation",
    "annotation-xml",
];
