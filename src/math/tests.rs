//! Tests for the typed MathML layer.
//!
//! Three kinds of check live here:
//!
//!  * the **subset pin** is derived from the validation constants at runtime (a `phf_map` lookup,
//!    not a copy of the names in this file), in both directions: the layer accepts every name the
//!    rules permit and rejects every name they do not, and the two tables it derives its operator
//!    and constant sets from are pinned against the same constants. A drift between the typed layer
//!    and the validator therefore fails the build;
//!  * the **behavioural checks** of the layer: the element order of the construction helpers, the
//!    navigation, the representability of the operators that are not built-in, the rendering of a
//!    node, the namespace declaration a formula built from scratch carries, and the round trip of a
//!    real document with a typed edit in it;
//!  * the **unit checks** of the value types (`MathOperator`, `MathConstant`, `MathSymbol`,
//!    `CnType`) against the same constants.
//!
//! Note for a later editor: this file *is* read by the creation scans in
//! `src/xml/xml_object/tests.rs`, which enumerate the places the crate builds an element from a
//! name/namespace pair. The element probes here are therefore built by parsing a document (which
//! creates no such site) rather than by `XmlElement::new_qualified`, and the nodes are built with
//! the construction helpers of the layer itself.

use crate::constants::element::{
    MATHML_ALLOWED_CHILDREN, MATHML_ALLOWED_DEFINITION_URLS, MATHML_ALLOWED_TYPES,
    MATHML_BINARY_OPERATORS, MATHML_UNARY_OPERATORS,
};
use crate::constants::namespaces::{URL_MATHML, URL_PACKAGE_QUAL, URL_SBML_CORE};
use crate::core::{Rule, RuleTypes};
use crate::math::*;
use crate::qual::FunctionTerm;
use crate::xml::{
    Document, OptionalXmlChild, OptionalXmlProperty, RequiredXmlChild, SbmlObject, XmlElement,
    XmlProperty, XmlWrapper,
};
use crate::Sbml;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// An empty document to build formula roots in.
fn probe() -> Document {
    Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8"?><probe/>"#)
        .expect("The probe document is well formed.")
}

/// An element with the given local name in the MathML namespace, produced by the parser.
///
/// The parser is what makes this helper free of any element-creation site, and it also ties the
/// probe to the way a document reaches the layer: everything below navigates elements of a parsed
/// tree.
fn math_element(name: &str) -> XmlElement {
    let source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><probe xmlns=\"{URL_MATHML}\"><{name}/></probe>"
    );
    let document = Document::parse_string(&source).expect("The probe document is well formed.");
    document
        .root()
        .expect("The probe has a root.")
        .child_elements()
        .into_iter()
        .next()
        .map(XmlElement::new)
        .expect("The probe has one child element.")
}

/// The serialization of a formula that contains `node` as its single expression.
///
/// The formula root is created the way a caller creates it (in the document the node belongs to), so
/// the result shows exactly the namespace declarations the layer writes.
fn formula(node: MathNode) -> String {
    let root = Math::new_empty(node.xml_element().document());
    root.set_expression(node);
    root.to_string()
}

/// The number of MathML namespace declarations in `source`.
fn mathml_declarations(source: &str) -> usize {
    source.matches(URL_MATHML).count()
}

// ---------------------------------------------------------------------------------------------
// One source of truth: the subset the validation layer enforces
// ---------------------------------------------------------------------------------------------

/// The typed layer recognizes exactly the names the validation rules permit, and no others.
///
/// The oracle is the crate's own constant, read at runtime; the negatives are the names the
/// operator tables know but the allowed-child list does not, which is the inconsistency recorded as
/// C16 (they can never be valid children of a `math` element, so the layer must reject them).
#[test]
fn the_typed_layer_recognizes_exactly_the_validated_names() {
    let allowed = MATHML_ALLOWED_CHILDREN["math"];
    assert_eq!(allowed.len(), 73, "The measured subset changed.");

    for name in allowed.iter().copied().chain(["math"]) {
        let node = MathNode::try_from_element(math_element(name));
        assert!(
            node.is_some(),
            "<{name}> is permitted by the validation rules but is not a typed node."
        );
    }

    let rejected = MATHML_UNARY_OPERATORS
        .iter()
        .chain(MATHML_BINARY_OPERATORS.iter())
        .filter(|it| !allowed.contains(it))
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rejected.len(),
        22,
        "The number of names the arity tables know and the allowed-child list does not changed."
    );
    for name in rejected {
        assert!(
            MathNode::try_from_element(math_element(name)).is_none(),
            "<{name}> is not a permitted child of a <math> element, so it must not be a node."
        );
    }

    // The layer has no scope-out list: everything the rules accept is typed. This assertion is what
    // keeps it that way, because a name added to the constant without a typed representation would
    // fail the loop above.
    let typed = MathOperator::ALL
        .iter()
        .map(|it| it.name())
        .chain(MathConstant::ALL.iter().map(|it| it.name()))
        .chain(STRUCTURAL_ELEMENT_NAMES.iter().copied())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        typed,
        allowed.iter().copied().collect::<BTreeSet<_>>(),
        "The typed layer and the validation constants disagree about the subset."
    );
}

/// The element names of the fifteen declared structural types, in the order this module declares
/// them.
///
/// The list is a *pin*, not an input: the check below proves that the declared types recognize
/// exactly these names, so a wrapper that is dropped or renamed fails here.
const STRUCTURAL_ELEMENT_NAMES: &[&str] = &[
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

/// The fifteen structural names are the allowed names that are neither an operator nor a constant,
/// and each of them is a *declared* node kind (not an operator or constant value).
#[test]
fn the_structural_names_are_the_declared_element_kinds() {
    let allowed = MATHML_ALLOWED_CHILDREN["math"];
    let operators = MathOperator::ALL
        .iter()
        .map(|it| it.name())
        .collect::<BTreeSet<_>>();
    let constants = MathConstant::ALL
        .iter()
        .map(|it| it.name())
        .collect::<BTreeSet<_>>();

    assert_eq!(operators.len(), 52, "The operator set changed.");
    assert_eq!(constants.len(), 6, "The constant set changed.");
    assert!(
        operators.is_disjoint(&constants),
        "An element name cannot be an operator and a constant."
    );
    assert!(operators.is_subset(&allowed.iter().copied().collect()));
    assert!(constants.is_subset(&allowed.iter().copied().collect()));

    let structural = allowed
        .iter()
        .copied()
        .filter(|it| !operators.contains(it) && !constants.contains(it))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        structural.len(),
        15,
        "The number of structural elements changed."
    );
    assert_eq!(
        structural,
        STRUCTURAL_ELEMENT_NAMES
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        "The structural element names changed."
    );

    // Every structural name is a declared kind, and every operator/constant name is its value.
    for name in structural {
        let kind = MathNode::try_from_element(math_element(name))
            .unwrap()
            .kind();
        assert!(
            kind.is_declared_element(),
            "<{name}> must be a declared element kind, found {kind:?}."
        );
        assert_eq!(kind.element_name(), name);
    }
    for operator in MathOperator::ALL {
        match MathNode::try_from_element(math_element(operator.name())).unwrap() {
            MathNode::Operator {
                operator: found, ..
            } => assert_eq!(found, *operator),
            other => panic!(
                "<{}> must be an operator node, found {}.",
                operator.name(),
                other.element_name()
            ),
        }
    }
    for constant in MathConstant::ALL {
        match MathNode::try_from_element(math_element(constant.name())).unwrap() {
            MathNode::Constant {
                constant: found, ..
            } => assert_eq!(found, *constant),
            other => panic!(
                "<{}> must be a constant node, found {}.",
                constant.name(),
                other.element_name()
            ),
        }
    }
}

/// The name tables of the three enumerations are injective and round trip.
#[test]
fn the_name_tables_of_the_enumerations_round_trip() {
    for operator in MathOperator::ALL {
        assert_eq!(MathOperator::from_name(operator.name()), Some(*operator));
        assert_eq!(operator.to_string(), operator.name());
    }
    for constant in MathConstant::ALL {
        assert_eq!(MathConstant::from_name(constant.name()), Some(*constant));
    }
    for number_type in CnType::ALL {
        assert_eq!(CnType::from_name(number_type.name()), Some(*number_type));
    }
    assert_eq!(
        MathOperator::ALL.len(),
        MathOperator::ALL.iter().collect::<HashSet<_>>().len(),
        "Two operators share a name."
    );
    assert_eq!(
        MathConstant::ALL.len(),
        MathConstant::ALL.iter().collect::<HashSet<_>>().len(),
        "Two constants share a name."
    );

    // A name outside the subset is not an operator, not a constant and not a number type.
    for name in ["gcd", "mean", "selector", "union", ""] {
        assert_eq!(MathOperator::from_name(name), None);
        assert_eq!(MathConstant::from_name(name), None);
        assert_eq!(CnType::from_name(name), None);
    }
}

/// The arity of an operator mirrors the tables rule 10218 reads, with the two exceptions named.
#[test]
fn the_operator_arity_mirrors_the_validation_tables() {
    let mut counts = (0usize, 0usize, 0usize, 0usize);
    for operator in MathOperator::ALL {
        let name = operator.name();
        match operator.arity() {
            MathArity::Unary => {
                counts.0 += 1;
                assert!(MATHML_UNARY_OPERATORS.contains(&name));
                assert!(!MATHML_BINARY_OPERATORS.contains(&name));
            }
            MathArity::Binary => {
                counts.1 += 1;
                assert!(MATHML_BINARY_OPERATORS.contains(&name));
                assert!(!MATHML_UNARY_OPERATORS.contains(&name));
            }
            MathArity::UnaryOrBinary => {
                counts.2 += 1;
                assert!(
                    name == "minus" || name == "root",
                    "only `minus` and `root` accept either count, found {name}"
                );
            }
            MathArity::Nary => {
                counts.3 += 1;
                assert!(!MATHML_UNARY_OPERATORS.contains(&name));
                assert!(!MATHML_BINARY_OPERATORS.contains(&name));
            }
        }
    }
    // `root` is the second member of the exception set; it is binary in the tables but may be given
    // one argument (its `degree` is optional).
    assert_eq!(MathOperator::Root.arity(), MathArity::UnaryOrBinary);
    assert_eq!(
        counts,
        (32, 5, 2, 13),
        "The arity classification of the operators changed."
    );
}

/// The four SBML symbols are exactly the four URLs rule 10205 permits.
#[test]
fn the_symbols_are_the_permitted_definition_urls() {
    let urls = MathSymbol::ALL
        .iter()
        .map(|it| it.definition_url())
        .collect::<Vec<_>>();
    assert_eq!(urls, MATHML_ALLOWED_DEFINITION_URLS);
    for symbol in MathSymbol::ALL {
        assert_eq!(MathSymbol::from_url(symbol.definition_url()), Some(*symbol));
        assert_eq!(symbol.to_string(), symbol.definition_url());
    }
    assert_eq!(
        MathSymbol::from_url("http://example.org/not-a-sbml-symbol"),
        None
    );
}

/// The four number types are exactly the four values rule 10207 permits, and the default is `real`.
#[test]
fn the_number_types_are_the_permitted_type_values() {
    let names = CnType::ALL
        .iter()
        .map(|it| it.name())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        MATHML_ALLOWED_TYPES
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(CnType::default(), CnType::Real);
    assert_eq!(CnType::try_from("real".to_string()), Ok(CnType::Real));
    assert_eq!(
        CnType::try_from("rational".to_string()),
        Ok(CnType::Rational)
    );
    let error = CnType::try_from("double".to_string()).expect_err("`double` is not a type.");
    assert!(
        error.contains("double"),
        "the message names the value: {error}"
    );
}

// ---------------------------------------------------------------------------------------------
// Construction, element order and navigation
// ---------------------------------------------------------------------------------------------

/// An `apply` writes its operator first and its arguments after it, in the order they are given.
#[test]
fn an_application_is_written_in_the_order_of_its_content_model() {
    let document = probe();
    let apply = MathNode::apply(
        document.clone(),
        MathOperator::Minus,
        [
            MathNode::ci(document.clone(), "x"),
            MathNode::real(document, 0.5),
        ],
    );
    assert_eq!(
        formula(apply),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><apply><minus/><ci>x</ci>\
             <cn type=\"real\">0.5</cn></apply></math>"
        )
    );
}

/// The four number forms write the `type` attribute, and a two-token number writes its `sep`.
#[test]
fn the_number_forms_write_their_type_and_separator() {
    let document = probe();
    let cases = [
        (MathNode::integer(document.clone(), -3), "integer", "-3"),
        (MathNode::real(document.clone(), 1.5), "real", "1.5"),
    ];
    for (node, number_type, payload) in cases {
        assert_eq!(
            node.to_string(),
            format!("<cn type=\"{number_type}\">{payload}</cn>")
        );
    }

    // A `rational` and an `e-notation` number are two tokens around a `sep`.
    for (node, number_type, payload, expected) in [
        (
            MathNode::rational(document.clone(), 1, 3),
            CnType::Rational,
            "13",
            "<cn type=\"rational\">1<sep/>3</cn>",
        ),
        (
            MathNode::e_notation(document.clone(), 1.5, -3),
            CnType::ENotation,
            "1.5-3",
            "<cn type=\"e-notation\">1.5<sep/>-3</cn>",
        ),
    ] {
        assert_eq!(node.to_string(), expected);
        match node {
            MathNode::Cn(cn) => {
                // The payload is the text of the element, with the `sep` element taken out.
                assert_eq!(cn.payload().replace("<sep/>", ""), payload);
                assert_eq!(cn.number_type(), number_type);
            }
            other => panic!("a number is a `cn` node, found {}", other.element_name()),
        }
    }

    // A `cn` without a `type` attribute is a `real` by default, and the raw form writes no type.
    let raw = MathNode::cn(document, "42");
    assert_eq!(raw.to_string(), "<cn>42</cn>");
    match raw {
        MathNode::Cn(cn) => {
            assert_eq!(cn.type_().get(), None);
            assert_eq!(cn.number_type(), CnType::Real);
        }
        other => panic!("a number is a `cn` node, found {}", other.element_name()),
    }
}

/// A `csymbol` denotes its symbol through `definitionURL`, and it is written the way the schema
/// describes the element.
#[test]
fn a_symbol_element_carries_its_definition_url() {
    let document = probe();
    let node = MathNode::csymbol(document.clone(), MathSymbol::RateOf, "S1");
    assert_eq!(
        node.to_string(),
        format!(
            "<csymbol definitionURL=\"{}\" encoding=\"text\">S1</csymbol>",
            MathSymbol::RateOf.definition_url()
        )
    );
    match &node {
        MathNode::Csymbol(csymbol) => {
            assert_eq!(csymbol.symbol(), "S1");
            assert_eq!(
                csymbol.definition_url().get(),
                Some(MathSymbol::RateOf.definition_url().to_string())
            );
            assert_eq!(csymbol.encoding().get(), Some("text".to_string()));
        }
        other => panic!(
            "a `csymbol` builder yields a `csymbol`, found {}",
            other.element_name()
        ),
    }

    // A package symbol is written with its own URL.
    let custom = MathNode::csymbol_with_url(document, "http://example.org/symbol?a=1&b=2", "s");
    assert!(
        custom
            .to_string()
            .contains("http://example.org/symbol?a=1&amp;b=2"),
        "the URL is escaped as XML text: {}",
        custom
    );
}

/// The `degree` of a `root` and the `logbase` of a `log` are written immediately after the
/// operator, which is where the content model puts them.
#[test]
fn the_qualified_operators_write_their_qualifier_first() {
    let document = probe();
    let radicand = MathNode::ci(document.clone(), "x");
    let degree = MathNode::integer(document.clone(), 3);
    assert_eq!(
        formula(MathNode::root(
            document.clone(),
            radicand.clone(),
            Some(degree)
        )),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><apply><root/><degree><cn type=\"integer\">3</cn>\
             </degree><ci>x</ci></apply></math>"
        )
    );
    assert_eq!(
        formula(MathNode::root(document.clone(), radicand.clone(), None)),
        format!("<math xmlns=\"{URL_MATHML}\"><apply><root/><ci>x</ci></apply></math>")
    );

    let base = MathNode::integer(document.clone(), 2);
    assert_eq!(
        formula(MathNode::log(document.clone(), radicand, Some(base))),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><apply><log/><logbase><cn type=\"integer\">2</cn>\
             </logbase><ci>x</ci></apply></math>"
        )
    );
}

/// A `lambda` declares its bound variables before its body, and a `piecewise` writes its cases
/// before its default case.
#[test]
fn the_containers_write_their_children_in_order() {
    let document = probe();
    let body = MathNode::apply(
        document.clone(),
        MathOperator::Times,
        [
            MathNode::ci(document.clone(), "x"),
            MathNode::ci(document.clone(), "y"),
        ],
    );
    let lambda = MathNode::lambda(document.clone(), &["x", "y"], body);
    assert_eq!(
        formula(lambda),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><lambda><bvar><ci>x</ci></bvar><bvar><ci>y</ci></bvar>\
             <apply><times/><ci>x</ci><ci>y</ci></apply></lambda></math>"
        )
    );

    assert_eq!(
        formula(MathNode::piecewise(
            document.clone(),
            [(
                MathNode::integer(document.clone(), 1),
                MathNode::apply(
                    document.clone(),
                    MathOperator::Gt,
                    [
                        MathNode::ci(document.clone(), "x"),
                        MathNode::integer(document.clone(), 0),
                    ],
                ),
            )],
            Some(MathNode::integer(document.clone(), 0)),
        )),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><piecewise><piece><cn type=\"integer\">1</cn>\
             <apply><gt/><ci>x</ci><cn type=\"integer\">0</cn></apply></piece>\
             <otherwise><cn type=\"integer\">0</cn></otherwise></piecewise></math>"
        )
    );
}

/// A `semantics` writes the annotated expression first and the annotations after it.
#[test]
fn a_semantics_element_writes_its_annotations_last() {
    let document = probe();
    let node = MathNode::semantics(
        document.clone(),
        MathNode::ci(document.clone(), "x"),
        [
            MathNode::annotation(document.clone(), "text/plain", "the value of x"),
            MathNode::annotation_xml(document.clone(), "application/rdf+xml"),
        ],
    );
    assert_eq!(
        formula(node),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><semantics><ci>x</ci>\
             <annotation encoding=\"text/plain\">the value of x</annotation>\
             <annotation-xml encoding=\"application/rdf+xml\"/></semantics></math>"
        )
    );
}

/// Navigation reads the tree: `children` gives the recognized children, `raw_children` every child
/// element, and `parent` walks back up.
#[test]
fn navigation_follows_the_element_tree() {
    let document = probe();
    let root = Math::new_empty(document);
    root.set_expression(MathNode::apply(
        root.xml_element().document(),
        MathOperator::Plus,
        [
            MathNode::ci(root.xml_element().document(), "x"),
            MathNode::integer(root.xml_element().document(), 1),
        ],
    ));

    let math_node = root.node();
    let apply = math_node
        .children()
        .into_iter()
        .next()
        .expect("one expression");
    assert_eq!(apply.element_name(), "apply");
    assert_eq!(apply.children().len(), 3, "the operator and two arguments");

    let apply_wrapper = match &apply {
        MathNode::Apply(apply) => apply.clone(),
        other => panic!(
            "the formula is an application, found {}",
            other.element_name()
        ),
    };
    assert_eq!(apply_wrapper.argument_count(), 2);
    assert!(matches!(
        apply_wrapper.operator(),
        Some(MathNode::Operator {
            operator: MathOperator::Plus,
            ..
        })
    ));
    let arguments = apply_wrapper.arguments();
    assert_eq!(arguments.len(), 2);
    assert!(matches!(&arguments[0], MathNode::Ci(ci) if ci.symbol() == "x"));
    assert!(matches!(&arguments[1], MathNode::Cn(cn) if cn.number_type() == CnType::Integer));

    // Parent navigation walks from a node back to the root.
    assert_eq!(arguments[0].parent().unwrap().element_name(), "apply");
    assert_eq!(apply.parent().unwrap().element_name(), "math");
    assert!(math_node.parent().is_none(), "the root is detached here");

    // An application node built from a document still navigates: the escape hatch is used by the
    // test below, which starts from a formula containing an operator outside the subset.
    let application = MathNode::try_from_element(math_element("apply")).expect("an `apply` node");
    assert_eq!(application.children().len(), 0);
}

/// A child element outside the subset is skipped by `children` but visible through `raw_children`.
#[test]
fn an_out_of_subset_child_stays_reachable() {
    let source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <math xmlns=\"{URL_MATHML}\"><apply><gcd/><ci>x</ci><ci>y</ci></apply></math>"
    );
    let document = Document::parse_string(&source).expect("A well-formed probe.");
    let math = Math::from(XmlElement::new(
        document.root().expect("The probe has a root."),
    ));
    let apply = math.expression().expect("The formula has an expression.");
    assert_eq!(apply.element_name(), "apply");

    // The typed view: the operator is out of the subset and the two `ci` arguments are not.
    let apply_wrapper = match &apply {
        MathNode::Apply(apply) => apply,
        other => panic!(
            "the formula is an application, found {}",
            other.element_name()
        ),
    };
    assert!(apply_wrapper.operator().is_none());
    assert_eq!(apply_wrapper.arguments().len(), 2);
    assert_eq!(apply.raw_children().len(), 3);
    assert_eq!(apply.raw_children()[0].tag_name(), "gcd");
    assert_eq!(apply.children().len(), 2, "the foreign operator is skipped");

    // The raw element is the escape hatch: its name and its parent are still there.
    let foreign = &apply.raw_children()[0];
    assert_eq!(foreign.tag_name(), "gcd");
    assert_eq!(foreign.namespace_url(), URL_MATHML);
    assert_eq!(foreign.parent().expect("a parent").tag_name(), "apply");
}

/// The three representability cases of the review: a `ci` and a `csymbol` as the operator of an
/// application, and an operator that is not a built-in name at all.
#[test]
fn a_function_call_is_representable_in_every_form() {
    let document = probe();

    // A call to a function defined by a `functionDefinition`.
    let ci_call = MathNode::apply_with(
        document.clone(),
        MathNode::ci(document.clone(), "f"),
        [MathNode::ci(document.clone(), "x")],
    );
    assert_eq!(
        formula(ci_call),
        format!("<math xmlns=\"{URL_MATHML}\"><apply><ci>f</ci><ci>x</ci></apply></math>")
    );

    // A call of one of the SBML symbols, `rateOf`.
    let csymbol_call = MathNode::apply_with(
        document.clone(),
        MathNode::csymbol(document.clone(), MathSymbol::RateOf, "S1"),
        [MathNode::ci(document.clone(), "S1")],
    );
    assert_eq!(
        formula(csymbol_call),
        format!(
            "<math xmlns=\"{URL_MATHML}\"><apply>\
             <csymbol definitionURL=\"{}\" encoding=\"text\">S1</csymbol><ci>S1</ci></apply></math>",
            MathSymbol::RateOf.definition_url()
        )
    );

    // An operator the subset does not contain (MathML defines `gcd`, SBML does not permit it): the
    // typed layer rejects it as a *node*, and the element itself is still there.
    let foreign = MathNode::try_from_element(math_element("gcd"));
    assert!(foreign.is_none());
    let foreign = math_element("gcd");
    assert_eq!(foreign.tag_name(), "gcd");
    assert_eq!(foreign.namespace_url(), URL_MATHML);
}

/// A `MathNode` is a checked view: `try_from_element` rejects a foreign namespace and a foreign
/// name, and `is_element` re-checks a node whose element was changed underneath it.
#[test]
fn a_node_is_a_checked_view_of_an_element() {
    // The right name in the wrong namespace is not a node — not even for the root.
    let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                  <math xmlns=\"http://www.sbml.org/sbml/level3/version2/core\"/>";
    let document = Document::parse_string(source).expect("A well-formed probe.");
    let root = XmlElement::new(document.root().expect("a root"));
    assert_eq!(root.tag_name(), "math");
    assert!(Math::matches_element(&root) == false);
    assert!(MathNode::try_from_element(root).is_none());

    // A node built by the layer is recognized by its own kind.
    let document = probe();
    let node = MathNode::integer(document, 7);
    assert!(node.is_element());
    assert_eq!(node.kind().element_name(), "cn");
    assert_eq!(
        MathNode::try_from_element(node.xml_element().clone())
            .unwrap()
            .kind(),
        node.kind()
    );
}

// ---------------------------------------------------------------------------------------------
// Rendering and namespaces
// ---------------------------------------------------------------------------------------------

/// `Display` renders from the element tree: the text is what the document holds, escaped as XML
/// requires, with the nested elements and the attributes in place.
#[test]
fn display_renders_the_element_tree() {
    // A formula whose text and attribute values contain the two characters that have to be escaped,
    // whose expression is a nested application, and whose annotation names a URL with a `&` in it.
    let source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <math xmlns=\"{URL_MATHML}\">\
         <semantics>\
         <apply><times/><ci>x</ci><apply><plus/><cn>1</cn><cn>2</cn></apply></apply>\
         <annotation encoding=\"text/plain\">a &lt; b &amp;&amp; c</annotation>\
         <annotation-xml encoding=\"application/x-tex?a=1&amp;b=2\"/>\
         </semantics>\
         </math>"
    );
    let document = Document::parse_string(&source).expect("A well-formed probe.");
    let root = XmlElement::new(document.root().expect("a root"));
    let node = MathNode::try_from_element(root).expect("the probe is a formula");

    let rendered = node.to_string();
    let expected = source
        .split_once('>')
        .expect("the declaration ends")
        .1
        .to_string();
    assert_eq!(
        rendered, expected,
        "Rendering a node has to reproduce the element exactly as the document holds it."
    );
    assert!(rendered.contains("a &lt; b &amp;&amp; c"), "{rendered}");
    assert!(
        rendered.contains("application/x-tex?a=1&amp;b=2"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<apply><plus/><cn>1</cn><cn>2</cn></apply>"),
        "{rendered}"
    );

    // Escaping is not a one-way street: what is rendered parses back into the same structure.
    let reparsed = Document::parse_string(&format!("<probe>{rendered}</probe>"))
        .expect("What the layer renders is well-formed XML.");
    let child = reparsed
        .root()
        .expect("a root")
        .child_elements()
        .into_iter()
        .next()
        .map(XmlElement::new)
        .expect("one child");
    let reparsed = MathNode::try_from_element(child).expect("the rendered formula is a formula");
    assert_eq!(reparsed.to_string(), rendered);
    match reparsed.children().into_iter().next() {
        Some(MathNode::Semantics(semantics)) => {
            let annotations = semantics.annotations();
            assert_eq!(annotations.len(), 2);
            assert!(matches!(&annotations[0], MathNode::Annotation(it)
                if it.payload() == "a < b && c"));
        }
        other => panic!("the probe is a `semantics` element, found {other:?}"),
    }
}

/// A formula built from scratch carries exactly one MathML namespace declaration, however deeply
/// nested its expression is.
#[test]
fn a_formula_built_from_scratch_declares_mathml_once() {
    let document = probe();

    // A deeply nested expression, built bottom up and attached last.
    let mut node = MathNode::integer(document.clone(), 1);
    for _ in 0..4 {
        node = MathNode::apply(
            document.clone(),
            MathOperator::Times,
            [node, MathNode::ci(document.clone(), "x")],
        );
    }
    assert_eq!(
        mathml_declarations(&node.to_string()),
        0,
        "A subtree that has not entered a document declares nothing yet."
    );

    let written = formula(node);
    assert_eq!(
        mathml_declarations(&written),
        1,
        "A formula built from scratch must carry exactly one MathML namespace declaration: {written}"
    );

    // The same holds for the other shapes, including the ones with a `sep` and an annotation.
    for node in [
        MathNode::rational(document.clone(), 1, 3),
        MathNode::lambda(
            document.clone(),
            &["x"],
            MathNode::ci(document.clone(), "x"),
        ),
        MathNode::semantics(
            document.clone(),
            MathNode::ci(document.clone(), "x"),
            [MathNode::annotation(document.clone(), "text/plain", "x")],
        ),
        MathNode::piecewise(
            document.clone(),
            [(
                MathNode::integer(document.clone(), 1),
                MathConstant::True.node(document.clone()),
            )],
            None,
        ),
    ] {
        let written = formula(node);
        assert_eq!(mathml_declarations(&written), 1, "{written}");
    }

    // A subtree that is *not* attached to a formula can be made self-contained explicitly, and
    // attaching a self-contained subtree to a declared root still leaves one declaration.
    let detached = MathNode::apply(
        document.clone(),
        MathOperator::Plus,
        [
            MathNode::ci(document.clone(), "x"),
            MathNode::integer(document, 1),
        ],
    );
    detached.ensure_namespace();
    assert_eq!(mathml_declarations(&detached.to_string()), 1);
    let written = formula(detached);
    assert_eq!(mathml_declarations(&written), 2, "{written}");
}

/// A formula whose document binds MathML under a prefix keeps working: the child that enters the
/// document gets the declaration it needs.
#[test]
fn a_prefixed_binding_gets_a_declaration_where_it_is_needed() {
    let source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <sbml xmlns=\"http://www.sbml.org/sbml/level3/version2/core\" level=\"3\" version=\"2\">\
         <model id=\"m\"><listOfInitialAssignments>\
         <initialAssignment symbol=\"p\"><m:math xmlns:m=\"{URL_MATHML}\"><m:apply/></m:math></initialAssignment>\
         </listOfInitialAssignments><listOfParameters><parameter id=\"p\" constant=\"true\"/></listOfParameters>\
         </model></sbml>"
    );
    let document = Sbml::read_str(&source).expect("A readable document.");
    let assignment = document
        .model()
        .get()
        .expect("a model")
        .initial_assignments()
        .get()
        .expect("the list")
        .get(0);
    let math = assignment.math().get().expect("the formula");
    assert_eq!(math.expression().unwrap().element_name(), "apply");

    // The `m:apply` child is in the MathML namespace through the prefix, so the typed layer reads
    // it; a new child that is attached to it inherits nothing, and is given the default declaration.
    let child = MathNode::ci(document.xml_element().document(), "p");
    math.push(child);
    let written = math.to_string();
    assert_eq!(
        mathml_declarations(&written),
        2,
        "the prefixed binding and the child's own declaration: {written}"
    );
    assert!(written.contains("<ci xmlns=\""), "{written}");
}

/// Reading a real document and writing it back preserves every formula in it byte for byte.
///
/// The library guarantees the *text* of a document, not its byte layout: the serializer writes the
/// attributes of an element in a canonical order and re-wraps its declarations, so a document that
/// was written by another tool is not necessarily reproduced byte for byte (measured once over
/// `test-inputs/`: 42 of its 43 documents are readable and **none** of them is reproduced byte for
/// byte, the closest ones differing in attribute order, quoting or line wrapping). What a formula is
/// made of, on the other hand, is text and elements, so
/// the formula itself survives unchanged — which is what this test pins, on a document whose five
/// formulas are all preserved. The whole-corpus counterpart of this check is the round-trip snapshot
/// of `scripts/full_corpus_parity.sh` (`test_suite_roundtrip.txt`).
#[test]
fn a_document_with_a_formula_keeps_its_formulas_byte_for_byte() {
    let path = "test-inputs/model.sbml";
    let original = std::fs::read_to_string(path).expect("The example document is readable.");
    let document = Sbml::read_path(path).expect("The example document parses.");
    let written = document.to_xml_string().expect("It serializes.");

    let formulas = formulas(&original);
    assert_eq!(formulas.len(), 5, "The example document has five formulas.");
    for formula in &formulas {
        assert_eq!(
            written.matches(formula.as_str()).count(),
            1,
            "A formula has to survive the read-write cycle unchanged: {formula}"
        );
    }

    // Writing a document that the library itself wrote is stable: the canonical form is a fixed
    // point, so a read-write cycle on it is byte-identical.
    let canonical = Sbml::read_str(&written)
        .expect("What the library writes parses.")
        .to_xml_string()
        .expect("It serializes.");
    assert_eq!(canonical, written);
}

/// The `<math …>…</math>` fragments of a document, in document order.
fn formulas(source: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("<math") {
        let rest_of = &rest[start..];
        let Some(end) = rest_of.find("</math>") else {
            break;
        };
        result.push(rest_of[..end + "</math>".len()].to_string());
        rest = &rest_of[end + "</math>".len()..];
    }
    result
}

/// A typed edit changes the edited element and nothing else.
///
/// The evidence is the serialized document: everything outside the edited element — the element
/// order, the whitespace and text nodes, the comments, the unrelated siblings — has to be
/// byte-identical, which is what replacing the old rendering of the element with the new one in the
/// old document shows.
#[test]
fn a_typed_edit_is_minimal() {
    let path = "test-inputs/test-qual/basic_example.xml";
    let before = Sbml::read_path(path)
        .expect("The example document is readable.")
        .to_xml_string()
        .expect("It serializes.");
    let document = Sbml::read_path(path).expect("The example document is readable.");

    // Pick a `cn` element whose rendering occurs exactly once, so the replacement below is
    // unambiguous.
    let target = document
        .xml_element()
        .recursive_child_elements_filtered(|it| Cn::matches_element(it))
        .into_iter()
        .find(|it| before.matches(&it.element().to_string()).count() == 1)
        .expect("The example document contains a `cn` whose rendering is unique.");
    let old_rendering = target.element().to_string();

    let node = MathNode::try_from_element(target).expect("a `cn` element is a node");
    let payload = match &node {
        MathNode::Cn(cn) => cn.payload(),
        other => panic!("`Cn::matches_element` accepted a {}", other.element_name()),
    };
    assert!(!payload.is_empty());

    // The typed edit: replace the payload of that one number.
    match &node {
        MathNode::Cn(cn) => cn.set_payload("424242"),
        _ => unreachable!("checked above"),
    }
    let new_rendering = node.to_string();
    assert_ne!(old_rendering, new_rendering);

    let after = document.to_xml_string().expect("It serializes.");
    assert_eq!(
        before.replace(&old_rendering, &new_rendering),
        after,
        "Only the edited element may change."
    );
}

// ---------------------------------------------------------------------------------------------
// The `sbml:units` attribute
// ---------------------------------------------------------------------------------------------

/// Writing units produces the prefixed attribute the specification requires, and reuses a
/// declaration that the document already has.
#[test]
fn writing_units_declares_the_sbml_prefix() {
    // From scratch: the prefix is declared on the element that carries the attribute.
    let document = probe();
    let node = MathNode::integer(document, 2);
    let cn = match &node {
        MathNode::Cn(cn) => cn,
        other => panic!("a number is a `cn` node, found {}", other.element_name()),
    };
    cn.units().set_some(&"mole".to_string());
    let written = node.to_string();
    assert!(written.contains("sbml:units=\"mole\""), "{written}");
    assert!(written.contains("xmlns:sbml="), "{written}");

    // And the written prefix resolves to the SBML namespace.
    let resolved = node
        .xml_element()
        .element()
        .resolve_attribute_name("sbml:units")
        .expect("The written attribute name resolves.");
    assert_eq!(
        resolved.namespace().expect("it has a namespace").uri(),
        URL_SBML_CORE
    );

    // A document that already binds the prefix does not get a second declaration.
    let source = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <sbml xmlns=\"http://www.sbml.org/sbml/level3/version2/core\" xmlns:sbml=\"{URL_SBML_CORE}\" \
         level=\"3\" version=\"2\"><model id=\"m\"><listOfParameters><parameter id=\"p\" constant=\"true\"/>\
         </listOfParameters><listOfInitialAssignments><initialAssignment symbol=\"p\">\
         <math xmlns=\"{URL_MATHML}\"><cn>1</cn></math></initialAssignment></listOfInitialAssignments>\
         </model></sbml>"
    );
    let document = Sbml::read_str(&source).expect("A readable document.");
    let cn = document
        .xml_element()
        .recursive_child_elements_filtered(|it| Cn::matches_element(it))
        .into_iter()
        .next()
        .expect("The document has a `cn` element.");
    let cn = MathNode::try_from_element(cn).expect("a `cn` is a node");
    match &cn {
        MathNode::Cn(cn) => cn.units().set_some(&"mole".to_string()),
        _ => unreachable!("checked above"),
    }
    let written = document.to_xml_string().expect("It serializes.");
    assert_eq!(
        written.matches("xmlns:sbml").count(),
        1,
        "An existing declaration is reused: {written}"
    );
    assert!(written.contains("sbml:units=\"mole\""), "{written}");
}

/// Reading units matches the attribute by local name, which is how the validation rules read it.
#[test]
fn reading_units_matches_the_local_name() {
    let document = probe();
    for source in [
        format!(
            "<cn xmlns=\"{URL_MATHML}\" xmlns:sbml=\"{URL_SBML_CORE}\" sbml:units=\"mM\">1</cn>"
        ),
        format!("<cn xmlns=\"{URL_MATHML}\" units=\"mM\">1</cn>"),
    ] {
        let probe_document =
            Document::parse_string(&source).expect("The probe is well-formed XML.");
        let element = XmlElement::new(probe_document.root().expect("a root"));
        let node = MathNode::try_from_element(element)
            .unwrap_or_else(|| panic!("the probe is a `cn`: {source}"));
        match node {
            MathNode::Cn(cn) => {
                assert_eq!(cn.units().get(), Some("mM".to_string()));
                assert!(cn.units().is_set());
            }
            other => panic!("the probe is a `cn`, found {}", other.element_name()),
        }
    }

    // A `cn` without the attribute has no units, and clearing them removes the attribute.
    let node = MathNode::integer(document, 1);
    match &node {
        MathNode::Cn(cn) => {
            assert_eq!(cn.units().get(), None);
            cn.units().set_some(&"mole".to_string());
            assert_eq!(cn.units().get(), Some("mole".to_string()));
            cn.units().clear().expect("clearing cannot fail");
            assert_eq!(cn.units().get(), None);
            assert!(!node.to_string().contains("units"));
        }
        other => panic!("a number is a `cn` node, found {}", other.element_name()),
    }

    // `ci` exposes the same attribute for *reading* only, because rule 10220 permits the attribute
    // on a `cn` only while rule 10313 checks it on a `ci`; there is deliberately no setter.
    let source = format!("<ci xmlns=\"{URL_MATHML}\">x</ci>");
    let probe_document = Document::parse_string(&source).expect("The probe is well-formed XML.");
    let element = XmlElement::new(probe_document.root().expect("a root"));
    let ci = match MathNode::try_from_element(element).expect("a `ci` node") {
        MathNode::Ci(ci) => ci,
        other => panic!("the probe is a `ci`, found {}", other.element_name()),
    };
    assert_eq!(ci.symbol(), "x");
    assert_eq!(ci.units(), None);
}

// ---------------------------------------------------------------------------------------------
// The formula sites of the object model
// ---------------------------------------------------------------------------------------------

/// A document with a formula at every site of the object model that has one.
///
/// Each formula is a single `ci` whose text names its site, so the payload of a site identifies it:
/// an accessor that answered with another site's formula (or with none) is visible in the collected
/// `(site, payload)` pairs instead of passing silently.
fn document_with_formulas() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" xmlns:qual="{URL_PACKAGE_QUAL}"
      level="3" version="2" qual:required="true">
  <model id="m">
    <listOfFunctionDefinitions>
      <functionDefinition id="f"><math xmlns="{URL_MATHML}"><ci>functionDefinition</ci></math></functionDefinition>
    </listOfFunctionDefinitions>
    <listOfParameters><parameter id="p" constant="true"/></listOfParameters>
    <listOfInitialAssignments>
      <initialAssignment symbol="p"><math xmlns="{URL_MATHML}"><ci>initialAssignment</ci></math></initialAssignment>
    </listOfInitialAssignments>
    <listOfRules>
      <algebraicRule><math xmlns="{URL_MATHML}"><ci>algebraicRule</ci></math></algebraicRule>
      <assignmentRule variable="p"><math xmlns="{URL_MATHML}"><ci>assignmentRule</ci></math></assignmentRule>
      <rateRule variable="p"><math xmlns="{URL_MATHML}"><ci>rateRule</ci></math></rateRule>
    </listOfRules>
    <listOfReactions>
      <reaction id="r"><listOfReactants/><listOfProducts/>
        <kineticLaw><math xmlns="{URL_MATHML}"><ci>kineticLaw</ci></math></kineticLaw>
      </reaction>
    </listOfReactions>
    <listOfConstraints>
      <constraint><math xmlns="{URL_MATHML}"><ci>constraint</ci></math></constraint>
    </listOfConstraints>
    <listOfEvents>
      <event id="e">
        <trigger><math xmlns="{URL_MATHML}"><ci>trigger</ci></math></trigger>
        <priority><math xmlns="{URL_MATHML}"><ci>priority</ci></math></priority>
        <delay><math xmlns="{URL_MATHML}"><ci>delay</ci></math></delay>
        <listOfEventAssignments>
          <eventAssignment variable="p"><math xmlns="{URL_MATHML}"><ci>eventAssignment</ci></math></eventAssignment>
        </listOfEventAssignments>
      </event>
    </listOfEvents>
    <qual:listOfQualitativeSpecies>
      <qual:qualitativeSpecies qual:id="q" qual:constant="false" qual:maxLevel="1"/>
    </qual:listOfQualitativeSpecies>
    <qual:listOfTransitions>
      <qual:transition qual:id="t">
        <qual:listOfOutputs>
          <qual:output qual:id="o" qual:qualitativeSpecies="q" qual:transitionEffect="assignmentLevel"/>
        </qual:listOfOutputs>
        <qual:listOfFunctionTerms>
          <qual:defaultTerm qual:resultLevel="0"/>
          <qual:functionTerm qual:resultLevel="1"><math xmlns="{URL_MATHML}"><ci>functionTerm</ci></math></qual:functionTerm>
        </qual:listOfFunctionTerms>
      </qual:transition>
    </qual:listOfTransitions>
  </model>
</sbml>"#
    )
}

/// The same sites, with no formula anywhere.
///
/// Every one of them is optional in the content model except the qual `functionTerm`, so a site that
/// has no `math` child is a document this layer has to read as "no formula" rather than as an error.
fn document_without_formulas() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" xmlns:qual="{URL_PACKAGE_QUAL}"
      level="3" version="2" qual:required="true">
  <model id="m">
    <listOfFunctionDefinitions><functionDefinition id="f"/></listOfFunctionDefinitions>
    <listOfParameters><parameter id="p" constant="true"/></listOfParameters>
    <listOfInitialAssignments><initialAssignment symbol="p"/></listOfInitialAssignments>
    <listOfRules>
      <algebraicRule/>
      <assignmentRule variable="p"/>
      <rateRule variable="p"/>
    </listOfRules>
    <listOfReactions>
      <reaction id="r"><listOfReactants/><listOfProducts/><kineticLaw/></reaction>
    </listOfReactions>
    <listOfConstraints><constraint/></listOfConstraints>
    <listOfEvents>
      <event id="e">
        <trigger/><priority/><delay/>
        <listOfEventAssignments><eventAssignment variable="p"/></listOfEventAssignments>
      </event>
    </listOfEvents>
  </model>
</sbml>"#
    )
}

/// The identifier a site's formula refers to, which is the payload that identifies the site.
fn payload_of(formula: &MathNode) -> String {
    let expression = formula
        .children()
        .into_iter()
        .next()
        .expect("a formula has a top-level expression");
    match expression {
        MathNode::Ci(ci) => ci.symbol(),
        other => panic!(
            "the formulas of the probe are `ci` elements, found {}",
            other.element_name()
        ),
    }
}

/// Every formula site of the object model exposes its formula as a typed node, and the raw `math()`
/// accessor still describes the same element.
#[test]
fn every_formula_site_exposes_the_typed_node() {
    let document = Sbml::read_str(&document_with_formulas()).expect("A readable document.");
    let model = document.model().get().expect("The document has a model.");
    let mut found = Vec::new();

    let function_definition = model.function_definitions().get().unwrap().get(0);
    let node = function_definition.math_node().expect("a formula");
    assert_eq!(
        function_definition.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("functionDefinition", payload_of(&node)));

    let initial_assignment = model.initial_assignments().get().unwrap().get(0);
    let node = initial_assignment.math_node().expect("a formula");
    assert_eq!(
        initial_assignment.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("initialAssignment", payload_of(&node)));

    for rule in model.rules().get().unwrap().iter() {
        let node = rule.math_node().expect("a formula");
        assert_eq!(rule.math().get().unwrap().xml_element(), node.xml_element());
        let site = match rule.cast() {
            RuleTypes::Algebraic(_) => "algebraicRule",
            RuleTypes::Assignment(_) => "assignmentRule",
            RuleTypes::Rate(_) => "rateRule",
            RuleTypes::Other(_) => panic!("the probe has no unknown rule kind"),
        };
        found.push((site, payload_of(&node)));
    }

    let kinetic_law = model
        .reactions()
        .get()
        .unwrap()
        .get(0)
        .kinetic_law()
        .get()
        .unwrap();
    let node = kinetic_law.math_node().expect("a formula");
    assert_eq!(
        kinetic_law.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("kineticLaw", payload_of(&node)));

    let constraint = model.constraints().get().unwrap().get(0);
    let node = constraint.math_node().expect("a formula");
    assert_eq!(
        constraint.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("constraint", payload_of(&node)));

    let event = model.events().get().unwrap().get(0);
    let trigger = event.trigger().get().unwrap();
    let node = trigger.math_node().expect("a formula");
    assert_eq!(
        trigger.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("trigger", payload_of(&node)));

    let priority = event.priority().get().unwrap();
    let node = priority.math_node().expect("a formula");
    assert_eq!(
        priority.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("priority", payload_of(&node)));

    let delay = event.delay().get().unwrap();
    let node = delay.math_node().expect("a formula");
    assert_eq!(
        delay.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("delay", payload_of(&node)));

    let assignment = event.event_assignments().get().unwrap().get(0);
    let node = assignment.math_node().expect("a formula");
    assert_eq!(
        assignment.math().get().unwrap().xml_element(),
        node.xml_element()
    );
    found.push(("eventAssignment", payload_of(&node)));

    // The qual `functionTerm` is the one site whose formula is required, so its accessor yields the
    // node itself: binding it to a `MathNode` is what makes the asymmetry a compile-time property.
    let terms = model
        .transitions()
        .get()
        .unwrap()
        .get(0)
        .function_terms()
        .get();
    let function_term = terms
        .iter()
        .find(|it| FunctionTerm::matches_element(it.xml_element()))
        .map(|it| FunctionTerm::from(it.xml_element().clone()))
        .expect("The probe has a function term.");
    let node: MathNode = function_term.math_node();
    assert_eq!(function_term.math().get().xml_element(), node.xml_element());
    found.push(("functionTerm", payload_of(&node)));

    // Every site had a distinct formula, and a site that answered with another site's formula (or
    // did not answer at all) changes this comparison.
    found.sort();
    assert_eq!(
        found,
        vec![
            ("algebraicRule", "algebraicRule".to_string()),
            ("assignmentRule", "assignmentRule".to_string()),
            ("constraint", "constraint".to_string()),
            ("delay", "delay".to_string()),
            ("eventAssignment", "eventAssignment".to_string()),
            ("functionDefinition", "functionDefinition".to_string()),
            ("functionTerm", "functionTerm".to_string()),
            ("initialAssignment", "initialAssignment".to_string()),
            ("kineticLaw", "kineticLaw".to_string()),
            ("priority", "priority".to_string()),
            ("rateRule", "rateRule".to_string()),
            ("trigger", "trigger".to_string()),
        ]
    );
}

/// A site whose object has no `math` child answers `None` instead of failing.
#[test]
fn a_site_without_a_formula_answers_none() {
    let document = Sbml::read_str(&document_without_formulas()).expect("A readable document.");
    let model = document.model().get().expect("The document has a model.");

    assert!(model
        .function_definitions()
        .get()
        .unwrap()
        .get(0)
        .math_node()
        .is_none());
    assert!(model
        .initial_assignments()
        .get()
        .unwrap()
        .get(0)
        .math_node()
        .is_none());
    for rule in model.rules().get().unwrap().iter() {
        assert!(
            rule.math_node().is_none(),
            "a rule without a formula has no node"
        );
    }
    let kinetic_law = model
        .reactions()
        .get()
        .unwrap()
        .get(0)
        .kinetic_law()
        .get()
        .unwrap();
    assert!(kinetic_law.math_node().is_none());
    assert!(model
        .constraints()
        .get()
        .unwrap()
        .get(0)
        .math_node()
        .is_none());

    let event = model.events().get().unwrap().get(0);
    assert!(event.trigger().get().unwrap().math_node().is_none());
    assert!(event.priority().get().unwrap().math_node().is_none());
    assert!(event.delay().get().unwrap().math_node().is_none());
    assert!(event
        .event_assignments()
        .get()
        .unwrap()
        .get(0)
        .math_node()
        .is_none());

    // The ten *optional* sites above are the ones with an `Option`; the qual `functionTerm` is not
    // among them, and a document that omits its `math` child is a document error rather than a
    // `None` (rule 20411), which is why its accessor returns a `MathNode`.
}

// ---------------------------------------------------------------------------------------------
// The typed layer against the documents of this repository
// ---------------------------------------------------------------------------------------------

/// What a walk over the formulas of a directory found.
#[derive(Debug, Default, PartialEq, Eq)]
struct Sweep {
    /// The documents whose text mentions a formula, i.e. the ones that were parsed.
    documents_with_math: usize,
    /// The documents that contain a formula but are not well-formed XML.
    unreadable: Vec<String>,
    /// The `math` elements that are formula roots.
    formulas: usize,
    /// The descendant elements of those formulas.
    elements: usize,
    /// The descendant elements that are *not* nodes of the typed layer, by `name [why]`.
    outside: BTreeMap<String, usize>,
}

/// Walk every formula of every document below `root`.
///
/// The file is only parsed when its text mentions a `math` element, which keeps the walk over the
/// whole test corpus cheap; a document that mentions one and does not parse is recorded as
/// unreadable rather than silently skipped.
fn sweep_math(root: &str) -> Sweep {
    let mut sweep = Sweep::default();
    let mut files = Vec::new();
    collect_documents(root, &mut files);
    files.sort();
    for path in files {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !source.contains("<math") {
            continue;
        }
        sweep.documents_with_math += 1;
        let Ok(document) = Sbml::read_str(&source) else {
            sweep.unreadable.push(path);
            continue;
        };
        let formulas = document
            .xml_element()
            .recursive_child_elements_filtered(|it| Math::matches_element(it));
        for formula in formulas {
            sweep.formulas += 1;
            walk_formula(&formula, &mut sweep);
        }
    }
    sweep
}

/// Count the descendant elements of `formula`, recording the ones outside the typed layer.
fn walk_formula(element: &XmlElement, sweep: &mut Sweep) {
    for child in element.child_elements() {
        sweep.elements += 1;
        if MathNode::try_from_element(child.clone()).is_none() {
            let reason = if child.namespace_url() == URL_MATHML {
                "in the MathML namespace, not permitted by SBML".to_string()
            } else {
                format!("in the namespace '{}'", child.namespace_url())
            };
            let key = format!("{} [{}]", child.tag_name(), reason);
            *sweep.outside.entry(key).or_default() += 1;
        }
        walk_formula(&child, sweep);
    }
}

/// Every document of `root` with an `.xml` or `.sbml` extension.
fn collect_documents(root: &str, result: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_documents(&path.to_string_lossy(), result);
        } else if path
            .extension()
            .is_some_and(|it| it == "xml" || it == "sbml")
        {
            result.push(path.to_string_lossy().to_string());
        }
    }
}

/// Every element of every formula of the example documents is a node of the typed layer.
///
/// This is the evidence that the subset the layer is pinned against is the subset real documents
/// use: the example documents of this repository contain no MathML that the layer cannot describe,
/// so there is no scope-out list to keep in sync (the module documentation says so, and this test
/// shows it is true of the documents we ship).
#[test]
fn the_example_documents_are_covered_by_the_typed_layer() {
    let sweep = sweep_math("test-inputs");
    println!("{sweep:?}");
    assert!(
        sweep.documents_with_math >= 21 && sweep.formulas >= 300 && sweep.elements >= 3700,
        "The example documents lost their formulas: {sweep:?}"
    );
    assert!(
        sweep.unreadable.is_empty(),
        "An example document that mentions a formula is not well-formed XML: {:?}",
        sweep.unreadable
    );
    assert!(
        sweep.outside.is_empty(),
        "An example document uses MathML outside the SBML subset: {:?}",
        sweep.outside
    );
}

/// The MathML of the syntactic test corpus that is *not* a node of the typed layer is exactly the
/// deliberate violation the corresponding test case is about.
///
/// The corpus is the collection of documents the validation layer is measured against, and it
/// contains, on purpose, documents that break the MathML rules. Read through the typed layer they
/// split into two classes, both measured here: an element of the MathML namespace that SBML does not
/// permit, and a MathML name written in the SBML namespace (the rule 10201 documents, where the
/// formula itself is in the wrong namespace).
///
/// The figures count *recognised* formulas only: the walk starts from the elements that
/// `Math::matches_element` accepts, so a document whose `<math>` is itself in another namespace (the
/// `10201-fail-01-04/11` class, where the `<math>` carries the SBML namespace) is counted as a
/// document but contributes no formula — which is why the set below has four entries and not six.
///
/// The test skips (and passes) when the corpus is not unpacked, like the crate's other corpus gates.
#[test]
fn the_corpus_math_outside_the_subset_is_the_deliberate_violation() {
    if !std::path::Path::new("syntactic").is_dir() {
        // The corpus is git-ignored, so this test has to be green on a checkout without it (the
        // crate's other corpus gates skip the same way); it says so instead of passing silently.
        println!(
            "SKIP: the syntactic test suite is not unpacked in ./syntactic, so the corpus sweep \
             has nothing to walk (download a release from \
             https://github.com/sbmlteam/sbml-test-suite/releases and extract its syntactic subset \
             there)."
        );
        return;
    }
    let sweep = sweep_math("syntactic");
    println!("{sweep:?}");
    assert!(
        sweep.documents_with_math >= 3600 && sweep.formulas >= 5000 && sweep.elements >= 25000,
        "The corpus walk lost its documents: {sweep:?}"
    );
    // Seven documents of the corpus mention a formula and are *not* well-formed XML: they are the
    // rule 01009/01011/01023 cases about XML that a parser refuses, so the walk cannot read them.
    assert_eq!(
        sweep.unreadable,
        vec![
            "syntactic/01009/01009-fail-01-02-sev2-l2v1.xml",
            "syntactic/01009/01009-fail-01-03-sev2-l2v1.xml",
            "syntactic/01009/01009-fail-01-05-sev2-l2v2.xml",
            "syntactic/01009/01009-fail-01-06-sev2-l2v3.xml",
            "syntactic/01009/01009-fail-01-07-sev2-l2v4.xml",
            "syntactic/01011/01011-fail-01-01-sev2-l2v1.xml",
            "syntactic/01023/01023-fail-01-01-sev2-l2v1.xml",
        ]
        .iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>(),
        "The set of unparsable corpus documents changed."
    );
    // Each entry is a *document* that breaks the specification, not a gap of the typed layer:
    //
    //  * `mean` is a MathML 2.0 element that SBML does not permit — the negative side of the
    //    73-name subset, exercised by the seven `10202-fail-01-01..07` cases (rule 10202).
    //  * `ci` and `times` are inside a correctly namespaced `math` whose children the document
    //    writes in the SBML namespace instead (the default namespace of the document is SBML) —
    //    the `10201-fail-01-09/10/15/16` cases, where rule 10201 requires the formula's content to
    //    be placed within the MathML namespace. The typed layer rejects them for that reason alone.
    assert_eq!(
        sweep.outside,
        BTreeMap::from([
            (
                "ci [in the namespace 'http://www.sbml.org/sbml/level2/version4']".to_string(),
                1
            ),
            (
                "ci [in the namespace 'http://www.sbml.org/sbml/level2/version5']".to_string(),
                1
            ),
            (
                "mean [in the MathML namespace, not permitted by SBML]".to_string(),
                7
            ),
            (
                "times [in the namespace 'http://www.sbml.org/sbml/level2/version4']".to_string(),
                1
            ),
            (
                "times [in the namespace 'http://www.sbml.org/sbml/level2/version5']".to_string(),
                1
            ),
        ]),
        "The MathML of the corpus that the typed layer rejects changed."
    );
}

/// A description of the structure of a node: the element name of each node of its subtree, with the
/// payload of the token elements.
fn describe(node: &MathNode) -> String {
    let children = node
        .children()
        .iter()
        .map(describe)
        .collect::<Vec<_>>()
        .join(",");
    let payload = match node {
        MathNode::Ci(it) => format!("={}", it.symbol()),
        MathNode::Cn(it) => format!("={}[{}]", it.payload(), it.number_type().name()),
        MathNode::Csymbol(it) => format!("={}", it.symbol()),
        _ => String::new(),
    };
    if children.is_empty() {
        format!("{}{}", node.element_name(), payload)
    } else {
        format!("{}{}({})", node.element_name(), payload, children)
    }
}

/// A formula built with the construction helpers, attached to a real document, written out and read
/// back is the same structure — the end-to-end path a caller takes.
#[test]
fn a_built_formula_survives_a_real_document() {
    let document =
        Sbml::read_path("test-inputs/Mukandavire2020.xml").expect("The example document parses.");
    let kinetic_law = document
        .model()
        .get()
        .expect("The document has a model.")
        .reactions()
        .get()
        .expect("The model has reactions.")
        .get(0)
        .kinetic_law()
        .get()
        .expect("The reaction has a kinetic law.");

    // A piecewise expression, built in the document of the site it is attached to.
    let handle = document.xml_element().document();
    let built = Math::new(
        handle.clone(),
        MathNode::piecewise(
            handle.clone(),
            [(
                MathNode::integer(handle.clone(), 1),
                MathNode::apply(
                    handle.clone(),
                    MathOperator::Gt,
                    [
                        MathNode::ci(handle.clone(), "S"),
                        MathNode::real(handle.clone(), 0.5),
                    ],
                ),
            )],
            Some(MathNode::real(handle.clone(), 0.0)),
        ),
    );
    let before = describe(&built.node());
    let written_formula = built.to_string();
    assert_eq!(
        mathml_declarations(&written_formula),
        1,
        "A built formula declares MathML once: {written_formula}"
    );
    assert_eq!(
        before,
        "math(piecewise(piece(cn=1[integer],apply(gt,ci=S,cn=0.5[real])),otherwise(cn=0[real])))"
    );

    kinetic_law.math().set(built);
    let written = document.to_xml_string().expect("The document serializes.");
    let reread = Sbml::read_str(&written).expect("What the library writes parses.");
    let formula = reread
        .model()
        .get()
        .expect("The written document has a model.")
        .reactions()
        .get()
        .expect("It has reactions.")
        .get(0)
        .kinetic_law()
        .get()
        .expect("It has a kinetic law.")
        .math()
        .get()
        .expect("It has the formula it was given.");

    assert_eq!(describe(&formula.node()), before);
    assert_eq!(
        formula.to_string(),
        written_formula,
        "The formula survives the round trip through the document byte for byte."
    );
}
