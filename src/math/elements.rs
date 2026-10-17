//! The declared MathML element types of the SBML subset.
//!
//! Every structural construct of the subset is a thin wrapper around one MathML element, declared
//! with [`#[sbml_object]`](crate::xml::SbmlObject) exactly like an SBML object. That declaration is
//! what makes the *checked* matching of a formula possible: [`Apply::matches_element`] answers
//! "is this element an `apply`?", and [`MathNode`] is built on top of those answers.
//!
//! The wrappers are deliberately shallow. They add only what the specification says about their
//! element: the typed accessors for the children the content model fixes (an `apply` has an
//! operator and arguments, a `piece` a value and a condition, a `bvar` a bound variable), and the
//! properties for the attributes the SBML subset gives a meaning to (`type` and `sbml:units` on a
//! `cn`, `definitionURL` on a `ci` or `csymbol`). Everything else — the arbitrary content of an
//! `annotation-xml`, an attribute the specification permits but SBML does not interpret — is
//! reachable through the raw [`XmlElement`], which is the escape hatch this module documents.
//!
//! The element order of the accessors is the order of the specification's content models, which is
//! also the order the construction helpers of [`MathNode`] write.

use crate::constants::namespaces::NS_MATHML;
use crate::math::{MathNode, SbmlUnits};
use crate::xml::{OptionalProperty, OptionalXmlProperty, SbmlObject, XmlElement, XmlWrapper};
use sbml_macros::XmlWrapper;

/// The child elements of `parent` that are instances of `T`, in document order.
///
/// The predicate is the declared identity of `T`, so a child in a foreign namespace (or a sibling
/// element that merely has the same local name) is not picked up.
fn typed_children<T: SbmlObject>(parent: &XmlElement) -> Vec<T> {
    parent
        .child_elements_filtered(|it| T::matches_element(it))
        .into_iter()
        .map(T::from)
        .collect()
}

/// The first child element of `parent` that is an instance of `T`.
fn typed_child<T: SbmlObject>(parent: &XmlElement) -> Option<T> {
    parent
        .child_elements()
        .into_iter()
        .find(|it| T::matches_element(it))
        .map(T::from)
}

/// The first child element of `parent` that is *not* an instance of `T`.
///
/// Used where a content model is "some `T`s, then something else" (`lambda` and `semantics`).
fn first_child_not<T: SbmlObject>(parent: &XmlElement) -> Option<XmlElement> {
    parent
        .child_elements()
        .into_iter()
        .find(|it| !T::matches_element(it))
}

/// The `value` of the `type` attribute of a MathML `cn` element.
///
/// The specification fixes four permissible values (prose Section 3.4.2); a missing attribute means
/// [`CnType::Real`], which is why that value is the [`Default`] of this enumeration. A `cn` whose
/// `type` is one of the four but spelled differently (or a fifth value) is a document that rule
/// 10207 reports; this type reports it as a conversion error instead of guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CnType {
    /// `real` — the default when the attribute is missing.
    Real,
    /// `integer`
    Integer,
    /// `rational` — two values separated by a `sep` element.
    Rational,
    /// `e-notation` — two values separated by a `sep` element.
    ENotation,
}

impl Default for CnType {
    /// The value a `cn` element has when it carries no `type` attribute (prose Section 3.4.2).
    fn default() -> Self {
        CnType::Real
    }
}

impl CnType {
    /// Every value of this enumeration, in declaration order.
    pub const ALL: &'static [Self] = &[
        CnType::Real,
        CnType::Integer,
        CnType::Rational,
        CnType::ENotation,
    ];

    /// The value of the `type` attribute that represents this type.
    pub const fn name(self) -> &'static str {
        match self {
            CnType::Real => "real",
            CnType::Integer => "integer",
            CnType::Rational => "rational",
            CnType::ENotation => "e-notation",
        }
    }

    /// The type whose `type` attribute value is this name, if it is one of the four.
    pub fn from_name(name: &str) -> Option<Self> {
        CnType::ALL.iter().copied().find(|it| it.name() == name)
    }
}

impl std::fmt::Display for CnType {
    /// Writes the value of the `type` attribute.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl From<CnType> for String {
    fn from(value: CnType) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for CnType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CnType::from_name(&value).ok_or_else(|| {
            format!(
                "The value '{value}' is not a permitted MathML 'type' of a 'cn' element. \
                 Permitted values are: {}",
                CnType::ALL
                    .iter()
                    .map(|it| it.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

impl crate::xml::XmlPropertyType for CnType {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => CnType::try_from(value.to_string()).map(Some),
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string())
    }
}

/// The MathML `apply` element: an operator applied to its arguments.
///
/// The first child element is the operator — a built-in operator element such as `<plus/>`, a
/// `csymbol` (for example the SBML `rateOf` symbol), a `ci` (a function defined by a
/// `functionDefinition`), or any other operator a package defines — followed by the arguments. The
/// element order is fixed by the content model, and the number of arguments an operator has to be
/// given is what rule 10218 checks (see [`MathOperator::arity`](crate::math::MathOperator::arity)).
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "apply", namespace = NS_MATHML)]
pub struct Apply(XmlElement);

impl Apply {
    /// The operator of this application, as a typed node.
    ///
    /// `None` when the element has no child at all, or when the first child is outside the subset
    /// (an operator SBML does not permit). The raw element is the first child of
    /// [`Apply::xml_element`] in either case.
    pub fn operator(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(0)
            .and_then(MathNode::try_from_element)
    }

    /// The arguments of this application, as typed nodes.
    ///
    /// The arguments are every child element after the operator, so an argument that is outside the
    /// subset is skipped here; use [`MathNode::raw_children`] (or the raw element) to see all of
    /// them. The count the specification cares about is instead
    /// [`Apply::argument_count`], which counts the elements the document actually has.
    pub fn arguments(&self) -> Vec<MathNode> {
        self.xml_element()
            .child_elements()
            .into_iter()
            .skip(1)
            .filter_map(MathNode::try_from_element)
            .collect()
    }

    /// The number of argument elements of this application, i.e. its child elements minus the
    /// operator.
    ///
    /// This is exactly the count rule 10218 compares against the arity of the operator. It counts
    /// elements, not typed nodes, so it is meaningful for an operator the subset does not contain as
    /// well.
    pub fn argument_count(&self) -> usize {
        self.xml_element().child_elements().len().saturating_sub(1)
    }
}

/// The MathML `bvar` element: the declaration of a bound variable inside a `lambda`.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "bvar", namespace = NS_MATHML)]
pub struct BVar(XmlElement);

impl BVar {
    /// The `ci` element that names the bound variable.
    ///
    /// The content model requires exactly this child; a `bvar` without one is reported by the
    /// validation layer, and the raw element is available here as well.
    pub fn variable(&self) -> Option<Ci> {
        typed_child(self.xml_element())
    }
}

/// The MathML `ci` element: a reference to an identifier.
///
/// In a formula of an SBML model, the payload of a `ci` is the identifier of a `species`, a
/// `parameter`, a `compartment`, … — or, in the first position of an `apply`, the identifier of a
/// `functionDefinition` (rules 10214 and 10215). The payload is read and written as *text*: the
/// document keeps the whitespace a caller writes, while [`Ci::symbol`] trims it, because that is
/// how the validation rules interpret the value.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "ci", namespace = NS_MATHML)]
pub struct Ci(XmlElement);

impl Ci {
    /// The identifier this element refers to.
    ///
    /// The value is the text content of the element with the surrounding whitespace removed, which
    /// is how the validation rules (10214, 10215, 10216) interpret it.
    pub fn symbol(&self) -> String {
        self.xml_element().interpreted_text()
    }

    /// Set the identifier this element refers to.
    ///
    /// The content of the element becomes a single text node holding `symbol`; any content it had
    /// (text, a comment, child elements) is replaced.
    pub fn set_symbol(&self, symbol: &str) {
        self.xml_element().set_text_content(symbol);
    }

    /// The value of the `definitionURL` attribute, which bundled MathML uses to identify a symbol.
    ///
    /// Rule 10204 permits the attribute on a `ci`; SBML does not give it a meaning of its own.
    pub fn definition_url(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "definitionURL")
    }

    /// The value of the SBML `units` attribute, if the document has one.
    ///
    /// The specification contradicts itself about this attribute: rule 10220 permits `units` on a
    /// `cn` only, while rule 10313 lists "the SBML `units` attribute on MathML `ci` elements" among
    /// the unit identifiers it checks. The crate implements both readings, so a document that
    /// writes the attribute here is read and reported as rule 10313 describes, **but this accessor
    /// is read-only**: writing it would produce a document the crate's own rule 10220 rejects. Use
    /// [`Cn::units`] to write units.
    ///
    /// The attribute is matched by its local name, which is how rules 10220, 10221, 10311 and 10313
    /// read it, so a document that writes it with a prefix (the spelling the specification requires)
    /// and one that writes it without a prefix are read the same way.
    pub fn units(&self) -> Option<String> {
        self.xml_element().get_attribute("units")
    }
}

/// The MathML `cn` element: a number literal.
///
/// The payload is a number whose interpretation is fixed by the `type` attribute ([`CnType`]);
/// a `rational` or `e-notation` payload is two numbers separated by a `sep` element. SBML adds the
/// `units` attribute, which attributes a unit to the number (prose Section 3.4.2).
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "cn", namespace = NS_MATHML)]
pub struct Cn(XmlElement);

impl Cn {
    /// The value of the `type` attribute; missing means [`CnType::Real`] in MathML, but this
    /// property reports `None` for a missing attribute, so that a caller can tell the two apart.
    pub fn type_(&self) -> OptionalProperty<'_, CnType> {
        OptionalProperty::new(self.xml_element(), "type")
    }

    /// The type of the number, with the default of a missing `type` attribute applied.
    pub fn number_type(&self) -> CnType {
        self.type_().get().unwrap_or_default()
    }

    /// The SBML `units` attribute of this number.
    ///
    /// The attribute is read by local name (as rules 10220, 10221 and 10311 read it) and written in
    /// the spelling the specification requires: `sbml:units`, with a prefix bound to the SBML
    /// namespace, because a MathML element's default namespace is MathML and an attribute never
    /// takes the default namespace anyway (prose Section 3.4.2). See [`SbmlUnits`].
    pub fn units(&self) -> SbmlUnits<'_> {
        SbmlUnits::new(self.xml_element())
    }

    /// The number as it is written in the document, with its internal whitespace and `sep` elements
    /// preserved (a rational or `e-notation` payload is two tokens).
    ///
    /// The value is the text content of the element. Reading it does not interpret the number: a
    /// consumer that needs a number has to apply the numeric type of the model itself.
    pub fn payload(&self) -> String {
        self.xml_element().text_content()
    }

    /// Replace the number with a single text node holding `payload`.
    ///
    /// This removes the `sep` element of a `rational` or `e-notation` number, so a caller that
    /// changes the payload of such a number writes the whole rewritten payload (including the
    /// separator) itself.
    pub fn set_payload(&self, payload: &str) {
        self.xml_element().set_text_content(payload);
    }
}

/// The MathML `csymbol` element: a symbol defined outside the formula, identified by a URL.
///
/// The four symbols of the SBML namespace are [`MathSymbol`](crate::math::MathSymbol); a package
/// can define more (rule 10205 permits a package to extend the list of URLs, and the rules about
/// `rateOf` — 10223 to 10225 — apply to the SBML symbol of that name).
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "csymbol", namespace = NS_MATHML)]
pub struct Csymbol(XmlElement);

impl Csymbol {
    /// The value of the required `definitionURL` attribute, or `None` if the document omits it.
    pub fn definition_url(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "definitionURL")
    }

    /// The value of the `encoding` attribute.
    ///
    /// Rule 10203 permits the attribute on a `csymbol`; the MathML schema makes it required (with
    /// the fixed value `text` for this element), while no SBML rule enforces that, so a document
    /// that omits it is read without an error here.
    pub fn encoding(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "encoding")
    }

    /// The text content of the element, i.e. the name of the symbol as it appears in the document.
    pub fn symbol(&self) -> String {
        self.xml_element().interpreted_text()
    }

    /// Replace the text content of the element with `symbol`.
    pub fn set_symbol(&self, symbol: &str) {
        self.xml_element().set_text_content(symbol);
    }
}

/// The MathML `degree` element: the optional degree of a `root`.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "degree", namespace = NS_MATHML)]
pub struct Degree(XmlElement);

impl Degree {
    /// The expression that gives the degree, as a typed node.
    pub fn value(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(0)
            .and_then(MathNode::try_from_element)
    }
}

/// The MathML `lambda` element: the definition of a function.
///
/// A `lambda` names its arguments as `bvar` children and ends with the expression that computes the
/// function value. It is permitted only inside the `math` element of a `functionDefinition` (rule
/// 10208), which is what makes a `functionDefinition` callable with an `apply` whose operator is a
/// `ci` (rule 10219).
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "lambda", namespace = NS_MATHML)]
pub struct Lambda(XmlElement);

impl Lambda {
    /// The bound variables of this function, in document order.
    pub fn variables(&self) -> Vec<BVar> {
        typed_children(self.xml_element())
    }

    /// The expression that computes the value of this function, as a typed node.
    ///
    /// The content model puts the expression after the bound variables, so this is the first child
    /// that is not a `bvar`. The raw element is the escape hatch when the expression is outside the
    /// subset.
    pub fn body(&self) -> Option<MathNode> {
        first_child_not::<BVar>(self.xml_element()).and_then(MathNode::try_from_element)
    }
}

/// The MathML `logbase` element: the optional base of a logarithm.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "logbase", namespace = NS_MATHML)]
pub struct Logbase(XmlElement);

impl Logbase {
    /// The expression that gives the base, as a typed node.
    pub fn value(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(0)
            .and_then(MathNode::try_from_element)
    }
}

/// The MathML `otherwise` element: the default of a `piecewise`.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "otherwise", namespace = NS_MATHML)]
pub struct Otherwise(XmlElement);

impl Otherwise {
    /// The value of this case, as a typed node.
    pub fn value(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(0)
            .and_then(MathNode::try_from_element)
    }
}

/// The MathML `piece` element: one case of a `piecewise`.
///
/// A `piece` is a value and the condition under which it applies, in that order.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "piece", namespace = NS_MATHML)]
pub struct Piece(XmlElement);

impl Piece {
    /// The value of this case, as a typed node.
    pub fn value(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(0)
            .and_then(MathNode::try_from_element)
    }

    /// The condition of this case, as a typed node.
    pub fn condition(&self) -> Option<MathNode> {
        self.xml_element()
            .get_child_at(1)
            .and_then(MathNode::try_from_element)
    }
}

/// The MathML `piecewise` element: a piecewise-defined expression.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "piecewise", namespace = NS_MATHML)]
pub struct Piecewise(XmlElement);

impl Piecewise {
    /// The cases of this expression, in document order.
    pub fn pieces(&self) -> Vec<Piece> {
        typed_children(self.xml_element())
    }

    /// The default case, if the expression has one.
    pub fn otherwise(&self) -> Option<Otherwise> {
        typed_child(self.xml_element())
    }
}

/// The MathML `semantics` element: an expression with annotations attached to it.
///
/// SBML permits `semantics` (and the two annotation elements it can contain) so that a formula can
/// carry its own markup, for example an RDF description of what an identifier means. The
/// annotations are content of the formula, not SBML objects, so their inner structure is reachable
/// through the raw element only.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "semantics", namespace = NS_MATHML)]
pub struct Semantics(XmlElement);

impl Semantics {
    /// The value of the `definitionURL` attribute, which MathML uses to identify the annotation
    /// scheme.
    ///
    /// Rule 10204 permits the attribute on a `semantics`.
    pub fn definition_url(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "definitionURL")
    }

    /// The annotated expression, as a typed node.
    ///
    /// The content model puts the expression first and the annotations after it, so this is the
    /// first child that is neither an `annotation` nor an `annotation-xml`.
    pub fn expression(&self) -> Option<MathNode> {
        let element = self.xml_element();
        element
            .child_elements()
            .into_iter()
            .find(|it| !Annotation::matches_element(it) && !AnnotationXml::matches_element(it))
            .and_then(MathNode::try_from_element)
    }

    /// The annotations of this expression, in document order, as typed nodes.
    ///
    /// Every entry is either [`MathNode::Annotation`] or [`MathNode::AnnotationXml`]; the content of
    /// an `annotation-xml` element is arbitrary markup and is not interpreted here.
    pub fn annotations(&self) -> Vec<MathNode> {
        self.xml_element()
            .child_elements()
            .into_iter()
            .filter(|it| Annotation::matches_element(it) || AnnotationXml::matches_element(it))
            .filter_map(MathNode::try_from_element)
            .collect()
    }
}

/// The MathML `sep` element: the separator of a two-token `cn` payload.
///
/// The element has no content and only exists between the two numbers of a `rational` or
/// `e-notation` literal, which is why the construction helpers of [`MathNode`] write it for the
/// caller.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "sep", namespace = NS_MATHML)]
pub struct Sep(XmlElement);

/// The MathML `annotation` element: a text annotation of a `semantics` element.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "annotation", namespace = NS_MATHML)]
pub struct Annotation(XmlElement);

impl Annotation {
    /// The value of the required `encoding` attribute, or `None` if the document omits it.
    ///
    /// (The MathML schema makes the attribute required; no SBML rule enforces that, so a missing
    /// attribute is reported as a missing value rather than as an error.)
    pub fn encoding(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "encoding")
    }

    /// The text of the annotation.
    pub fn payload(&self) -> String {
        self.xml_element().text_content()
    }

    /// Replace the text of the annotation with `payload`.
    ///
    /// The content of the element becomes a single text node; a comment or child element that the
    /// annotation had is removed. A CDATA payload is not text in the XML data model, so it stays
    /// untouched by this accessor and remains visible through the raw element.
    pub fn set_payload(&self, payload: &str) {
        self.xml_element().set_text_content(payload);
    }
}

/// The MathML `annotation-xml` element: an XML annotation of a `semantics` element.
#[derive(Clone, Debug, XmlWrapper)]
#[sbml_object(name = "annotation-xml", namespace = NS_MATHML)]
pub struct AnnotationXml(XmlElement);

impl AnnotationXml {
    /// The value of the required `encoding` attribute, or `None` if the document omits it.
    pub fn encoding(&self) -> OptionalProperty<'_, String> {
        OptionalProperty::new(self.xml_element(), "encoding")
    }
}
