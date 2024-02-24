use std::ops::Deref;
use std::str::FromStr;
use xml_doc::{Document, Element};

use crate::constants::element::{
    MATHML_ALLOWED_CHILDREN_BY_ATTR, MATHML_ALLOWED_DEFINITION_URLS, MATHML_ALLOWED_TYPES,
    MATHML_BINARY_OPERATORS, MATHML_NARY_OPERATORS, MATHML_UNARY_OPERATORS,
};
use crate::constants::namespaces::URL_MATHML;
use crate::core::validation::get_allowed_children;
use crate::core::{BaseUnit, FunctionDefinition, KineticLaw, Math, Model};
use crate::xml::{RequiredXmlProperty, XmlWrapper};
use crate::{SbmlIssue, SbmlIssueSeverity};

impl Math {
    /// ### Applies rules:
    ///  - **[10201](Math::apply_rule_10201)** - MathML content is permitted only within [Math] element.
    ///  - **[10202](Math::apply_rule_10202)** - Validates list of permitted elements within [Math] element.
    ///  - **[10203](Math::apply_rule_10203)** - Ensures *encoding* attribute correct placement.
    ///  - **[10204](Math::apply_rule_10204)** - Ensures *definitionURL* attribute correct placement.
    ///  - **[10205](Math::apply_rule_10205)** - Ensures *definitionURL* attribute correct value.
    ///  - **[10206](Math::apply_rule_10206)** - Ensures *type* attribute correct placement.
    ///  - **[10207](Math::apply_rule_10207)** - Ensures *type* attribute correct value.
    ///  - **[10208](Math::apply_rule_10208)** - Validates *lambda* element usage.
    ///  - **[10214](Math::apply_rule_10214)** - Validates first *ci* element usage outside of [FunctionDefinition].
    ///  - **[10215](Math::apply_rule_10215)** - Validates non-first *ci* element usage outside of [FunctionDefinition].
    ///  - **[10216](Math::apply_rule_10216)** - Validates [LocalParameter](crate::core::LocalParameter) *id* occurrence.
    ///  - **[10218](Math::apply_rule_10218)** - Validates number of arguments for operators.
    ///  - **[10219](Math::apply_rule_10219)** - Validates number of arguments for [FunctionDefinition].
    ///  - **[10220](Math::apply_rule_10220)** - Ensures *units* attribute correct placement.
    ///  - **[10221](Math::apply_rule_10220)** - Ensures *units* attribute correct value.
    ///  - **[10223](Math::apply_rule_10223)** - Validates *rateOf* *csymbol* element has single argument.
    ///  - **[10224](Math::apply_rule_10224)** - Validates the argument of *rateOf* *csymbol* element.
    ///  - **[10225](Math::apply_rule_10225)** - Validates the value of argument of *rateOf* *csymbol* element.
    ///
    /// ### Ignored rules as of SBML Level 3 Version 1 Core:
    /// - **10209** - "The arguments of the MathML logical operators and, not, or, and xor must evaluate to Boolean values."
    /// - **10210** - "The arguments to the following MathML constructs must evaluate to numeric values (more specifically, they
    /// must evaluate to MathML real, integer, rational, or "e-notation" numbers, or the time, delay, avogadro, csymbol elements): abs,
    /// arccosh, arccos, arccoth, arccot, arccsch, arccsc, arcsech, arcsec, arcsinh, arcsin, arctanh, arctan, ceiling, cosh, cos, coth,
    /// cot, csch, csc, divide, exp, factorial, floor, ln, log, minus, plus, power, root, sech, sec, sinh, sin, tanh, tan, and times."
    /// - **10211** - "The values of all arguments to MathML eq and neq operators must evaluate to the same type, either all
    /// Boolean or all numeric."
    /// - **10212** - "The types of the values within MathML piecewise operators should all be consistent; i.e., the set of expressions
    /// that make up the first arguments of the piece and otherwise operators within the same piecewise operator should all return
    /// values of the same type."
    /// - **10213** - "The second argument of a MathML piece operator must evaluate to a Boolean value."
    /// - **10217** - "The MathML formulas in the following elements must yield numeric values (that is, MathML real, integer
    /// or "e-notation" numbers, or the time, delay, avogadro, or rateOf csymbol): math in KineticLaw, math in InitialAssignment, math in
    /// AssignmentRule, math in RateRule, math in AlgebraicRule, math in Event Delay, and math in EventAssignment."
    pub(crate) fn validate(&self, issues: &mut Vec<SbmlIssue>) {
        self.apply_rule_10201(issues);
        self.apply_rule_10202(issues);
        self.apply_rule_10203(issues);
        self.apply_rule_10204(issues);
        self.apply_rule_10205(issues);
        self.apply_rule_10206(issues);
        self.apply_rule_10207(issues);
        self.apply_rule_10208(issues);
        self.apply_rule_10214(issues);
        self.apply_rule_10215(issues);
        self.apply_rule_10216(issues);
        self.apply_rule_10218(issues);
        self.apply_rule_10219(issues);
        self.apply_rule_10220(issues);
        self.apply_rule_10221(issues);
        self.apply_rule_10223(issues);
        self.apply_rule_10224(issues);
        self.apply_rule_10225(issues);
    }

    /// ### Rule 10201
    /// is *partially* satisfied by the implementation of the rule
    /// [10102](crate::core::validation::apply_rule_10102) as we check each
    /// element present for its allowed children (except [Math] element that is
    /// the subject of this validation procedure) and thus **MathML** content
    /// can be present only within a [Math] element. However, additional check for
    /// explicit or implicit valid namespace of a [Math] element must be performed.
    fn apply_rule_10201(&self, issues: &mut Vec<SbmlIssue>) {
        if self.namespace_url() != URL_MATHML {
            issues.push(SbmlIssue {
                element: self.raw_element(),
                message: format!(
                    "Wrong namespace usage in a math element. Found {0}, but {1} should be used.",
                    self.namespace_url(),
                    URL_MATHML
                ),
                rule: "10201".to_string(),
                severity: SbmlIssueSeverity::Error,
            });
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10202
    /// Validates that only allowed subset of **MathML** child elements are present within [Math]
    /// element. An SBML package may allow new MathML elements to be added to this list, and if so,
    /// the package must define **required="true"** on the SBML container element
    /// [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10202(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children = self.raw_element().children_recursive(doc.deref());
        let allowed_children = get_allowed_children(self.xml_element());

        for child in children {
            if let Some(child_element) = child.as_element() {
                let child_tag_name = child_element.name(doc.deref());

                if !allowed_children.contains(&child_tag_name) {
                    issues.push(SbmlIssue {
                        element: child_element,
                        message: format!(
                            "Unknown child <{0}> of element <{1}>.",
                            child_tag_name, "math"
                        ),
                        rule: "10202".to_string(),
                        severity: SbmlIssueSeverity::Error,
                    });
                }
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10203
    /// In the SBML subset of MathML 2.0, the MathML attribute **encoding** is only permitted on
    /// **csymbol**, **annotation** and **annotation-xml**. No other MathML elements may have
    /// an **encoding** attribute. An SBML package may allow the encoding attribute on other
    /// elements, and if so, the package must define **required="true"** on the SBML container
    /// element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10203(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["encoding"];
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "encoding").is_some())
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [encoding] found on element <{0}>, which is forbidden. \
                        Attribute [encoding] is only permitted on <csymbol>, <annotation> and <annotation-xml>.",
                        name
                    ),
                    rule: "10203".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10204
    /// In the SBML subset of MathML 2.0, the MathML attribute **definitionURL** is only permitted
    /// on **ci**, **csymbol** and **semantics**. No other MathML elements may have a
    /// **definitionURL** attribute. An SBML package may allow the definitionURL attribute on other
    /// elements, and if so, the package must define **required="true"** on the SBML container
    /// element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10204(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["definitionURL"];
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "definitionURL").is_some())
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [definitionURL] found on element <{0}>, which is forbidden. \
                        Attribute [definitionURL] is only permitted on <ci>, <csymbol> and <semantics>.",
                        name
                    ),
                    rule: "10204".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10205
    /// In SBML Level 3, the only values permitted for the attribute **definitionURL** on a
    /// **csymbol** are "**http://www.sbml.org/sbml/symbols/time**",
    /// "**http://www.sbml.org/sbml/symbols/delay**",
    /// "**http://www.sbml.org/sbml/symbols/avogadro**", and
    /// "**http://www.sbml.org/sbml/symbols/rateOf**". An SBML package may allow new values for the
    /// definitionURL attribute of a csymbol, and if so, the package must define **required="true"**
    /// on the SBML container element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10205(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| {
                child.attribute(doc.deref(), "definitionURL").is_some()
                    && child.name(doc.deref()) == "csymbol"
            })
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let value = child.attribute(doc.deref(), "definitionURL").unwrap();
            if !MATHML_ALLOWED_DEFINITION_URLS.contains(&value) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Invalid definitionURL value found '{0}'. Permitted values are: {1}",
                        value,
                        MATHML_ALLOWED_DEFINITION_URLS
                            .iter()
                            .map(|url| url.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    rule: "10205".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10206
    /// In the SBML subset of MathML 2.0, the MathML attribute **type** is only permitted on the
    /// **cn**
    /// construct. **No** other MathML elements may have a type attribute. An SBML package may allow
    /// the type attribute on other elements, and if so, the package must define **required="true"**
    /// on the SBML container element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10206(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "type").is_some())
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["type"].contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [type] found on element <{0}>, which is forbidden. \
                        Attribute [type] is only permitted on <cn>.",
                        name
                    ),
                    rule: "10204".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10207
    /// The only permitted values for the attribute **type** on MathML cn elements are
    /// "**e-notation**", "**real**", "**integer**", and "**rational**". An SBML package may
    /// allow new values for the type attribute, and if so, the package must define
    /// **required="true"** on the SBML container element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10207(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "type").is_some())
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let value = child.attribute(doc.deref(), "type").unwrap();

            if !MATHML_ALLOWED_TYPES.contains(&value) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Invalid type value found '{0}'. Permitted values are: \
                    'e-notation', 'real', 'integer' and 'rational'",
                        value
                    ),
                    rule: "10206".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10208
    /// MathML **lambda** elements are only permitted as either the first element inside the
    /// [**Math**] element of a [**FunctionDefinition**](FunctionDefinition) object,
    /// or as the first element of a **semantics** element immediately inside the [**Math**] element
    /// of a [**FunctionDefinition**](FunctionDefinition) object. MathML **lambda**
    /// elements may not be used elsewhere in an SBML model. An SBML package may allow **lambda**
    /// elements on other elements, and if so, the package must define **required="true"** on the
    /// SBML container element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10208(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.name(doc.deref()) == "lambda")
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let parent = child.parent(doc.deref()).unwrap();
            let parent_name = parent.name(doc.deref());

            if parent_name == "math" {
                let grandparent = parent.parent(doc.deref()).unwrap();
                Self::validate_lambda_placement(doc.deref(), child, parent, grandparent, issues);
            } else if parent_name == "semantics" {
                let grandparent = parent.parent(doc.deref()).unwrap();
                let top_parent = grandparent.parent(doc.deref()).unwrap();
                Self::validate_lambda_placement(doc.deref(), child, parent, top_parent, issues);
            } else {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Invalid immediate parent of <lambda>. Only <math> and <semantics> are \
                        valid immediate parents. Actual parent: <{0}>",
                        parent_name
                    ),
                    rule: "10208".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
        }
    }

    /// Checks if:
    ///  1. top-level parent of **lambda** is a [**FunctionDefinition**](FunctionDefinition).
    ///  2. **lambda** is the first child of its immediate parent
    fn validate_lambda_placement(
        doc: &Document,
        child: Element,
        parent: Element,
        toplevel_parent: Element,
        issues: &mut Vec<SbmlIssue>,
    ) {
        if toplevel_parent.name(doc) != "functionDefinition" {
            // the (great)grandparent of <lambda> must be <functionDefinition>
            issues.push(SbmlIssue {
                element: child,
                message: format!(
                    "A <lambda> found in invalid scope of <{0}>. \
                The <lambda> can be located only within <functionDefinition> (in <math>).",
                    toplevel_parent.name(doc)
                ),
                rule: "10208".to_string(),
                severity: SbmlIssueSeverity::Error,
            });
        } else if *parent.child_elements(doc).first().unwrap() != child {
            // the <lambda> must be the first child inside <math> (or <semantics>)
            issues.push(SbmlIssue {
                element: child,
                message: "The <lambda> must be the first element within <math>.".to_string(),
                rule: "10208".to_string(),
                severity: SbmlIssueSeverity::Error,
            })
        }
    }

    /// ### Rule 10214
    /// Outside of a [**FunctionDefinition**](FunctionDefinition) object, if a MathML
    /// **ci** element is the first element within a MathML apply element, then the **ci** element's
    /// value can only be chosen from the set of identifiers of
    /// [**FunctionDefinition**](FunctionDefinition) objects defined in the enclosing
    /// SBML [Model](crate::core::model) object.
    pub(crate) fn apply_rule_10214(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let parent_name = self
            .raw_element()
            .parent(doc.deref())
            .unwrap()
            .name(doc.deref());

        if parent_name != "functionDefinition" {
            let children_of_interest = self
                .raw_element()
                .child_elements_recursive(doc.deref())
                .iter()
                .filter(|child| {
                    child.name(doc.deref()) == "apply"
                        && child
                            .child_elements(doc.deref())
                            .first()
                            .unwrap()
                            .name(doc.deref())
                            == "ci"
                })
                .copied()
                .collect::<Vec<Element>>();

            let identifiers = Model::for_child_element(self.document(), self.xml_element())
                .unwrap()
                .function_definition_identifiers();

            for child in children_of_interest {
                let value = match child.child_elements(doc.deref()).first() {
                    Some(element) => element.text_content(doc.deref()),
                    None => "".to_string(),
                };

                if !identifiers.contains(&value) {
                    issues.push(SbmlIssue {
                        element: child,
                        message: format!(
                            "Function '{0}' not defined. \
                            Function referred by <ci> must be defined in <functionDefinition> object \
                            with relevant identifier (id).",
                            value
                        ),
                        rule: "10214".to_string(),
                        severity: SbmlIssueSeverity::Error,
                    })
                }
            }
        }
    }

    // TODO: create an issue "the identifiers of [LocalParameter](crate::core::reaction::LocalParameter) objects that are [Reaction](crate::core::reaction::Reaction) in which the [FunctionDefinition] appears (if it appears inside the [Math] object of a [KineticLaw])"
    // TODO: add comment about "any identifiers (in the SId namespace of the model) belonging to an object class defined by an SBML Level 3 package as having mathematical meaning." to existing issue about adding extensions/packages
    /// ### Rule 10215
    /// Outside of a [FunctionDefinition] object, if a MathML **ci** element is not the first element within
    /// a MathML **apply**, then the **ci** element's value may only be chosen from the following set of
    /// identifiers: the identifiers of [Species](crate::core::species::Species),
    /// [Compartment](crate::core::compartment::Compartment), [Parameter](crate::core::parameter::Parameter),
    /// [SpeciesReference](crate::core::reaction::SpeciesReference) and [Reaction]
    /// objects defined in the enclosing [Model] object; the identifiers of
    /// [LocalParameter](crate::core::reaction::LocalParameter) objects that are children of the
    /// [Reaction](crate::core::reaction::Reaction) in which the [FunctionDefinition] appears (if it appears inside
    /// the [Math] object of a [KineticLaw]); and any identifiers (in the SId namespace of the model) belonging to an
    /// object class defined by an SBML Level 3 package as having mathematical meaning.
    pub(crate) fn apply_rule_10215(&self, issues: &mut Vec<SbmlIssue>) {
        let is_out_of_function_definition =
            FunctionDefinition::for_child_element(self.document(), self.xml_element()).is_none();

        if is_out_of_function_definition {
            let doc = self.read_doc();
            let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
            let identifiers = [
                model.species_reference_identifiers(),
                model.compartment_identifiers(),
                model.parameter_identifiers(),
                model.species_identifiers(),
                model.species_reference_identifiers(),
                model.reaction_identifiers(),
                model.local_parameter_identifiers(), // TODO: include only localParameters whose kineticLaw::math element contains some functionDefinition id
            ]
            .concat();
            let apply_elements = self
                .raw_element()
                .child_elements_recursive(doc.deref())
                .iter()
                .filter(|child| child.name(doc.deref()) == "apply")
                .copied()
                .collect::<Vec<Element>>();

            for apply in apply_elements {
                let ci_elements = apply
                    .child_elements(doc.deref())
                    .iter()
                    .filter(|child| child.name(doc.deref()) == "ci")
                    .skip(1)
                    .copied()
                    .collect::<Vec<Element>>();

                for ci in ci_elements {
                    let value = ci.text_content(doc.deref());

                    if !identifiers.contains(&value) {
                        issues.push(SbmlIssue {
                            element: ci,
                            message: format!(
                                "Invalid identifier value '{0}' in <ci>. Identifier not found.",
                                value
                            ),
                            rule: "10215".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        })
                    }
                }
            }
        }
    }

    /// ### Rule 10216
    /// The id attribute value of a [LocalParameter] object defined within a [KineticLaw] object may only be
    /// used, in core, in MathML ci elements within the math element of that same [KineticLaw]; in other
    /// words, the identifier of the [LocalParameter] object is not visible to other parts of the model outside
    /// of that [Reaction] instance. In package constructs, the **id** attribute value of a [LocalParameter] object
    /// may only be used in MathML ci elements or as the target of an SIdRef attribute if that package
    /// construct is a child of the parent [Reaction].
    pub(crate) fn apply_rule_10216(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let all_local_param_ids = model.local_parameter_identifiers();
        let scoped_local_param_ids =
            match KineticLaw::for_child_element(self.document(), self.xml_element()) {
                Some(k) => k.local_parameter_identifiers(),
                None => vec![],
            };
        let b_variables = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.name(doc.deref()) == "bvar")
            .map(|bvar| {
                bvar.child_elements(doc.deref())
                    .first()
                    .unwrap()
                    .text_content(doc.deref())
            })
            .collect::<Vec<String>>();
        let ci_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.name(doc.deref()) == "ci")
            .copied()
            .collect::<Vec<Element>>();

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());
            if !b_variables.contains(&value)
                && all_local_param_ids.contains(&value)
                && !scoped_local_param_ids.contains(&value)
            {
                issues.push(SbmlIssue {
                    element: ci,
                    message: format!("A <localParameter> identifier '{0}' found out of scope of its <KineticLaw>",
                                     value),
                    rule: "10216".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            }
        }
    }

    /// ### Rule 10218
    /// A MathML operator must be supplied the number of arguments appropriate for that operator.
    pub(crate) fn apply_rule_10218(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let apply_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.name(doc.deref()) == "apply")
            .copied()
            .collect::<Vec<Element>>();

        for apply in apply_elements {
            let children = apply.child_elements(doc.deref());
            let child_count = children.len();

            // iterate through children of an <apply> element
            for child in children {
                let name = child.name(doc.deref());

                if MATHML_UNARY_OPERATORS.contains(&name) {
                    // <minus> is allowed to have 1 OR 2 arguments
                    if name == "minus" && child_count - 1 != 1 && child_count - 1 != 2 {
                        issues.push(SbmlIssue {
                            element: child,
                            message: format!(
                                "Invalid number ({0}) of arguments for operator <minus>. \
                                The operator <minus> can take either 1 or 2 arguments.",
                                child_count - 1
                            ),
                            rule: "10218".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        })
                    } else if child_count - 1 != 1 && name != "minus" {
                        issues.push(SbmlIssue {
                            element: child,
                            message: format!(
                                "Invalid number ({0}) of arguments for unary operator <{1}>",
                                child_count - 1,
                                name
                            ),
                            rule: "10218".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        });
                    }
                } else if MATHML_BINARY_OPERATORS.contains(&name) {
                    // root is allowed to have 1 OR 2 arguments
                    if name == "root" && child_count - 1 != 1 && child_count - 1 != 2 {
                        issues.push(SbmlIssue {
                            element: child,
                            message: format!(
                                "Invalid number ({0}) of arguments for operator <root>. \
                                The operator <root> can take either 1 or 2 arguments.",
                                child_count - 1
                            ),
                            rule: "10218".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        })
                    } else if child_count - 1 != 2 && name != "root" {
                        issues.push(SbmlIssue {
                            element: child,
                            message: format!(
                                "Invalid number ({0}) of arguments for binary operator <{1}>.",
                                child_count - 1,
                                name
                            ),
                            rule: "10218".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        });
                    }
                } else if MATHML_NARY_OPERATORS.contains(&name) && child_count - 1 == 0 {
                    issues.push(SbmlIssue {
                        element: child,
                        message: format!("An N-ary operator <{0}> with 0 arguments found. Use of N-ary operators without any arguments is discouraged.", name),
                        rule: "10218".to_string(),
                        severity: SbmlIssueSeverity::Warning,
                    });
                }
            }
        }
    }

    /// ### Rule 10219
    /// The number of arguments used in a call to a function defined by a [FunctionDefinition] object must
    /// equal the number of arguments accepted by that function, if defined. In other words, it must equal
    /// the number of MathML **bvar** elements inside the **lambda** element of the function definition, if
    /// present.
    pub(crate) fn apply_rule_10219(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();

        let apply_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.name(doc.deref()) == "apply")
            .copied()
            .collect::<Vec<Element>>();

        for apply in apply_elements {
            let children = apply.child_elements(doc.deref());
            let child_count = children.len() as i32;
            let function_call = children.first();

            if function_call.is_some() && function_call.unwrap().name(doc.deref()) == "ci" {
                let function_call = function_call.unwrap();
                let func_identifiers = model.function_definition_identifiers();
                let id = function_call.text_content(doc.deref());

                if func_identifiers.contains(&id) {
                    let Some(expected_args) = model.function_definition_arguments(id.as_str())
                    else {
                        continue;
                    };

                    if child_count - 1 != expected_args as i32 {
                        issues.push(SbmlIssue {
                            element: *function_call,
                            message: format!(
                                "Invalid number of arguments ({0}) provided for function '{1}'. \
                                The function '{2}' takes {3} arguments.",
                                child_count - 1,
                                id,
                                id,
                                expected_args
                            ),
                            rule: "10219".to_string(),
                            severity: SbmlIssueSeverity::Error,
                        });
                    }
                }
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10220
    /// The SBML attribute **units** may only be added to MathML **cn** elements; no other MathML elements
    /// are permitted to have the **units** attribute. An SBML package may allow the **units** attribute
    /// on other elements, and if so, the package must define **required="true"** on the SBML container
    /// element [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10220(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest: Vec<Element> = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| child.attribute(doc.deref(), "units").is_some())
            .copied()
            .collect();

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["units"].contains(&name) {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Attribute [units] found on element <{0}>, which is forbidden. \
                        Attribute [units] is only permitted on <cn>.",
                        name
                    ),
                    rule: "10220".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
        }
    }

    /// ### Rule 10221
    /// The value of the SBML attribute units on a MathML cn element must be chosen from either the
    /// set of identifiers of UnitDefinition objects in the model, or the set of base units defined by SBML.
    pub(crate) fn apply_rule_10221(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let unit_identifiers = Model::for_child_element(self.document(), self.xml_element())
            .unwrap()
            .unit_definition_identifiers();
        let cn_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| {
                child.name(doc.deref()) == "cn" && child.attribute(doc.deref(), "units").is_some()
            })
            .copied()
            .collect::<Vec<Element>>();

        for cn in cn_elements {
            let value = cn.attribute(doc.deref(), "units").unwrap();

            if !unit_identifiers.contains(&value.to_string()) && BaseUnit::from_str(value).is_err()
            {
                issues.push(SbmlIssue {
                    element: cn,
                    message: format!(
                        "Invalid unit identifier '{0}' found. \
                        Only identifiers of <unitDefinition> objects and base units can be used in <cn>.",
                        value
                    ),
                    rule: "10221".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
        }
    }

    /// ### Rule 10223
    /// The single argument for the *rateOf* **csymbol** function must be a **ci** element.
    pub(crate) fn apply_rule_10223(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| {
                child.name(doc.deref()) == "apply" && !child.child_elements(doc.deref()).is_empty()
            })
            .filter(|apply| {
                apply
                    .child_elements(doc.deref())
                    .first()
                    .unwrap()
                    .attribute(doc.deref(), "definitionURL")
                    .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf")
            })
            .copied()
            .collect::<Vec<Element>>();

        for child in children_of_interest {
            let apply_children = child.child_elements(doc.deref());

            if apply_children.len() != 2 {
                issues.push(SbmlIssue {
                    element: child,
                    message: format!(
                        "Invalid number ({0}) of arguments provided for rateOf <csymbol>. \
                         The call of rateOf <csymbol> must have precisely one argument.",
                        apply_children.len() - 1
                    ),
                    rule: "10223".to_string(),
                    severity: SbmlIssueSeverity::Error,
                });
            } else {
                let argument_name = apply_children.last().unwrap().name(doc.deref());

                if argument_name != "ci" {
                    issues.push(SbmlIssue {
                        element: child,
                        message: format!(
                            "Invalid argument <{0}> provided for <csymbol>.\
                             The rateOf <csymbol> must have <ci> as its only argument.",
                            argument_name
                        ),
                        rule: "10223".to_string(),
                        severity: SbmlIssueSeverity::Error,
                    })
                }
            }
        }
    }

    /// ### Rule 10224
    /// The target of a *rateOf* **csymbol** function must not appear as the *variable* of an
    /// [AssignmentRule](crate::core::rule::AssignmentRule), nor may its value be determined by an
    /// [AlgebraicRule](crate::core::rule::AlgebraicRule).
    pub(crate) fn apply_rule_10224(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let ci_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .iter()
            .filter(|child| {
                child.name(doc.deref()) == "apply"
                    && child.child_elements(doc.deref()).len() > 1
                    && child
                        .child_elements(doc.deref())
                        .first()
                        .unwrap()
                        .attribute(doc.deref(), "definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf")
                    && child
                        .child_elements(doc.deref())
                        .get(1)
                        .unwrap()
                        .name(doc.deref())
                        == "ci"
            })
            .copied()
            .collect::<Vec<Element>>();
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_rule_determinants = model.algebraic_rule_ci_values();

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());

            if assignment_rule_variables.contains(&value) {
                issues.push(SbmlIssue {
                    element: ci,
                    message:
                    format!("The value of target ('{0}') of rateOf <csymbol> found as a variable of <assignmentRule>.",
                            value),
                    rule: "10224".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
                // TODO: what does "determined by algebraicRule" mean and how to check it?
                // TODO: same as 10225
            } else if algebraic_rule_determinants.contains(&value) {
                issues.push(SbmlIssue {
                    element: ci,
                    message:
                    format!("The value of target ('{0}') of rateOf <csymbol> determined by an <algebraicRule>.",
                            value),
                    rule: "10224".to_string(),
                    severity: SbmlIssueSeverity::Error,
                })
            }
        }
    }

    /// ### Rule 10225
    /// If the target of a *rateOf* **csymbol** function is a [Species](crate::core::species::Species) with a
    /// *hasOnlySubstanceUnits* value of *"false"*, the **compartment** of that [Species](crate::core::species::Species)
    /// must not appear as the *variable* of an [AssignmentRule](crate::core::rule::AssignmentRule),
    /// nor may its *size* be determined by an [AlgebraicRule](crate::core::rule::AlgebraicRule).
    pub(crate) fn apply_rule_10225(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_ci_values = model.algebraic_rule_ci_values();
        let ci_elements = self
            .raw_element()
            .child_elements_recursive(doc.deref())
            .into_iter()
            .filter(|child| {
                child.name(doc.deref()) == "apply"
                    && child.child_elements(doc.deref()).len() > 1
                    && child
                        .child_elements(doc.deref())
                        .first()
                        .unwrap()
                        .attribute(doc.deref(), "definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf")
                    && child
                        .child_elements(doc.deref())
                        .get(1)
                        .unwrap()
                        .name(doc.deref())
                        == "ci"
            })
            .collect::<Vec<Element>>();

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());

            if let Some(species) = model.find_species(value.as_str()) {
                if !species.has_only_substance_units().get() {
                    if let Some(compartment) =
                        model.find_compartment(species.compartment().get().as_str())
                    {
                        let compartment_id = compartment.id().get();

                        if assignment_rule_variables.contains(&compartment_id) {
                            issues.push(SbmlIssue {
                                element: ci,
                                message: format!(
                                    "The <compartment> with id '{0}' found as the [variable] of an <assignmentRule>.",
                                    compartment_id
                                ),
                                rule: "10225".to_string(),
                                severity: SbmlIssueSeverity::Error,
                            })
                        } else if !compartment.constant().get()
                            && algebraic_ci_values.contains(&compartment_id)
                        {
                            issues.push(SbmlIssue {
                                element: ci,
                                message: format!(
                                    "The <compartment>'s size with id '{0}' is possible to determine by an <algebraicRule>.",
                                    compartment_id
                                ),
                                rule: "10225".to_string(),
                                severity: SbmlIssueSeverity::Error,
                            })
                        }
                    }
                }
            }
        }
    }
}
