use crate::constants::element::{
    MATHML_ALLOWED_CHILDREN_BY_ATTR, MATHML_ALLOWED_DEFINITION_URLS, MATHML_ALLOWED_TYPES,
    MATHML_BINARY_OPERATORS, MATHML_UNARY_OPERATORS,
};
use crate::constants::namespaces::URL_MATHML;
use crate::core::validation::{apply_rule_10313, get_allowed_children, matches_unit_sid_pattern};
use crate::core::{BaseUnit, FunctionDefinition, KineticLaw, Math, Model};
use crate::xml::{RequiredXmlProperty, XmlElement, XmlWrapper};
use crate::SbmlIssue;
use std::str::FromStr;

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
    ///  - **[10214](Math::apply_rule_10214)** - Validates first *ci* element usage outside [FunctionDefinition].
    ///  - **[10215](Math::apply_rule_10215)** - Validates non-first *ci* element usage outside [FunctionDefinition].
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
    /// must evaluate to MathML real, integer, rational, or "e-notation" numbers, or the time, delay, avogadro, csymbol elements): `abs`,
    /// `arccosh`, `arccos`, `arccoth`, `arccot`, `arccsch`, `arccsc`, `arcsech`, `arcsec`, `arcsinh`, `arcsin`, `arctanh`, `arctan`, `ceiling`,
    /// `cosh`, `cos`, `coth`, `cot`, `csch`, `csc`, `divide`, `exp`, `factorial`, `floor`, `ln`, `log`, `minus`, `plus`, `power`, `root`,
    /// `sech`, `sec`, `sinh`, `sin`, `tanh`, `tan`, and `times`."
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
        self.apply_rule_10311(issues);
        self.apply_rule_10313(issues);
    }

    /// ### Rule 10201
    /// This rule is *partially* satisfied by the implementation of the rule
    /// [10102](crate::core::validation::apply_rule_10102_and_derivatives) as we check each
    /// element present for its allowed children (except [Math] element that is
    /// the subject of this validation procedure) and thus **MathML** content
    /// can be present only within a [Math] element. However, additional check for
    /// explicit or implicit valid namespace of a [Math] element must be performed.
    ///
    /// TODO:
    ///     This condition is never triggered, because when the `math` element has the wrong
    ///     namespace, the `Math` object is never created and thus cannot be validated. And since
    ///     `math` elements are typically optional, this does not create any
    fn apply_rule_10201(&self, issues: &mut Vec<SbmlIssue>) {
        let namespace = self.namespace_url();
        if namespace != URL_MATHML {
            let message = format!("Wrong namespace usage in a `math` element. Found `{namespace}`, but `{URL_MATHML}` should be used.");
            issues.push(SbmlIssue::new_error("10201", self, message));
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10202
    /// Validates that only the allowed subset of **MathML** child elements are present within
    /// a [Math] element. An SBML package may allow new MathML elements to be added to this list,
    /// and if so, the package must define **required="true"** on the SBML container element
    /// [**sbml**](crate::Sbml).
    pub(crate) fn apply_rule_10202(&self, issues: &mut Vec<SbmlIssue>) {
        let allowed_children = get_allowed_children(self.xml_element());

        for child in self.recursive_child_elements() {
            let child_tag_name = child.tag_name();
            if !allowed_children.contains(&child_tag_name.as_str()) {
                let message = format!("Unknown child <{child_tag_name}> of element <math>.");
                issues.push(SbmlIssue::new_error("10202", &child, message));
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
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["encoding"];
        let relevant_children =
            self.recursive_child_elements_filtered(|it| it.has_attribute("encoding"));

        for child in relevant_children {
            let name = child.tag_name();
            if !allowed.contains(&name.as_str()) {
                let message = format!(
                    "Attribute [encoding] found on element <{name}>, which is forbidden. \
                        Attribute [encoding] is only permitted on <csymbol>, <annotation> and <annotation-xml>."
                );
                issues.push(SbmlIssue::new_error("10203", &child, message));
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
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["definitionURL"];
        let relevant_children =
            self.recursive_child_elements_filtered(|it| it.has_attribute("definitionURL"));

        for child in relevant_children {
            let name = child.tag_name();
            if !allowed.contains(&name.as_str()) {
                let message = format!(
                    "Attribute [definitionURL] found on element <{name}>, which is forbidden. \
                        Attribute [definitionURL] is only permitted on <ci>, <csymbol> and <semantics>."
                );
                issues.push(SbmlIssue::new_error("10204", &child, message));
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
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "csymbol" && child.has_attribute("definitionURL")
        });

        for child in children_of_interest {
            // Unwrap is safe, because we only consider children where the attribute is set.
            let value = child.get_attribute("definitionURL").unwrap();
            if !MATHML_ALLOWED_DEFINITION_URLS.contains(&value.as_str()) {
                let message = format!(
                    "Invalid definitionURL value found '{}'. Permitted values are: {:?}",
                    value, MATHML_ALLOWED_DEFINITION_URLS
                );
                issues.push(SbmlIssue::new_error("10205", &child, message));
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
        let children_of_interest =
            self.recursive_child_elements_filtered(|child| child.has_attribute("type"));

        for child in children_of_interest {
            let name = child.tag_name();
            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["type"].contains(&name.as_str()) {
                let message = format!(
                    "Attribute [type] found on element <{name}>, which is forbidden. \
                        Attribute [type] is only permitted on <cn>."
                );
                issues.push(SbmlIssue::new_error("10206", &child, message));
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
        let children_of_interest =
            self.recursive_child_elements_filtered(|child| child.has_attribute("type"));

        for child in children_of_interest {
            let value = child.get_attribute("type").unwrap();

            if !MATHML_ALLOWED_TYPES.contains(&value.as_str()) {
                let message = format!(
                    "Invalid type value found '{value}'. Permitted values are: \
                    'e-notation', 'real', 'integer' and 'rational'"
                );
                issues.push(SbmlIssue::new_error("10207", &child, message));
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
        let children_of_interest =
            self.recursive_child_elements_filtered(|child| child.tag_name() == "lambda");

        for child in children_of_interest {
            // The parent must exist, because these are children of this math element.
            // Furthermore, we also assume that if the parent is `math`, then it must have
            // a parent, and if it is `semantics`, then its parent must have a parent as well.
            // This should be a reasonable assumption for any SBML document that is valid-enough
            // to get to this point.
            let parent = child.parent().unwrap();
            let parent_name = parent.tag_name();

            if parent_name == "math" {
                let grandparent = parent.parent().unwrap();
                Self::validate_lambda_placement(child, parent, grandparent, issues);
            } else if parent_name == "semantics" {
                let grandparent = parent.parent().unwrap();
                let top_parent = grandparent.parent().unwrap();
                Self::validate_lambda_placement(child, parent, top_parent, issues);
            } else {
                let message = format!(
                    "Invalid immediate parent of <lambda>. Only <math> and <semantics> are \
                        valid immediate parents. Actual parent: <{parent_name}>"
                );
                issues.push(SbmlIssue::new_error("10208", &child, message));
            }
        }
    }

    /// Checks if:
    ///  1. top-level parent of **lambda** is a [**FunctionDefinition**](FunctionDefinition).
    ///  2. **lambda** is the first child of its immediate parent
    fn validate_lambda_placement(
        child: XmlElement,
        parent: XmlElement,
        toplevel_parent: XmlElement,
        issues: &mut Vec<SbmlIssue>,
    ) {
        let toplevel_parent = toplevel_parent.tag_name();
        if toplevel_parent != "functionDefinition" {
            // the (great)grandparent of <lambda> must be <functionDefinition>
            let message = format!(
                "A <lambda> found in invalid scope of <{toplevel_parent}>. \
                The <lambda> can be located only within <functionDefinition> (in <math>)."
            );
            issues.push(SbmlIssue::new_error("10208", &child, message));
            return;
        }

        let is_first_child = parent
            .get_child_at(0)
            .map(|it| it.raw_element() == child.raw_element())
            .unwrap_or(false);

        if !is_first_child {
            // the <lambda> must be the first child inside <math> (or <semantics>)
            let message = "The <lambda> must be the first element within <math>.".to_string();
            issues.push(SbmlIssue::new_error("10208", &child, message));
        }
    }

    /// ### Rule 10214
    /// Outside a [FunctionDefinition] object, if a MathML **ci** element is the first element
    /// within a MathML apply element, then the **ci** element's value can only be chosen from
    /// the set of identifiers of [FunctionDefinition] objects defined in the enclosing
    /// SBML [Model] object.
    pub(crate) fn apply_rule_10214(&self, issues: &mut Vec<SbmlIssue>) {
        let parent_name = self.parent().unwrap().tag_name();

        if parent_name != "functionDefinition" {
            let children_of_interest = self.recursive_child_elements_filtered(|child| {
                child.tag_name() == "apply" && {
                    child
                        .get_child_at(0)
                        .map(|it| it.tag_name() == "ci")
                        .unwrap_or(false)
                }
            });

            let identifiers = Model::for_child_element(self.xml_element())
                .unwrap()
                .function_definition_identifiers();

            for child in children_of_interest {
                // This unwrap must succeed because we enforced that ci is the first child.
                let value = child.get_child_at(0).unwrap().text_content();

                if !identifiers.contains(&value) {
                    let message = format!(
                        "Function '{value}' not defined. \
                            Function referred by <ci> must be defined in <functionDefinition> object \
                            with relevant identifier (id)."
                    );
                    issues.push(SbmlIssue::new_error("10214", &child, message));
                }
            }
        }
    }

    /// ### Rule 10215
    /// Outside a [FunctionDefinition] object, if a MathML **ci** element is not the first element within
    /// a MathML **apply**, then the **ci** element's value may only be chosen from the following set of
    /// identifiers: the identifiers of [Species], [Compartment], [Parameter], [SpeciesReference]
    /// and [Reaction] objects defined in the enclosing [Model] object; the identifiers of
    /// [LocalParameter] objects that are children of the [Reaction] in which the
    /// [FunctionDefinition] appears (if it appears inside the [Math] object of a [KineticLaw]),
    /// and any identifiers (in the SId namespace of the model) belonging to an
    /// object class defined by an SBML Level 3 package as having mathematical meaning.
    pub(crate) fn apply_rule_10215(&self, issues: &mut Vec<SbmlIssue>) {
        let is_out_of_function_definition =
            FunctionDefinition::for_child_element(self.xml_element()).is_none();

        if !is_out_of_function_definition {
            return;
        }

        let model = Model::for_child_element(self.xml_element()).unwrap();
        let identifiers = [
            model.species_identifiers(),
            model.compartment_identifiers(),
            model.parameter_identifiers(),
            model.species_reference_identifiers(),
            model.reaction_identifiers(),
            model.local_parameter_identifiers(),
        ]
        .concat();

        let apply_elements =
            self.recursive_child_elements_filtered(|child| child.tag_name() == "apply");

        for apply in apply_elements {
            let ci_elements = apply
                .child_elements()
                .into_iter()
                .skip(1)
                .filter(|child| child.tag_name() == "ci")
                .collect::<Vec<_>>();

            for ci in ci_elements {
                let value = ci.text_content();

                if !identifiers.contains(&value) {
                    let message = format!(
                        "Invalid identifier value '{value}' in <ci>. Identifier not found."
                    );
                    issues.push(SbmlIssue::new_error("10215", &ci, message));
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
        let model = Model::for_child_element(self.xml_element()).unwrap();
        let all_local_param_ids = model.local_parameter_identifiers();

        let scoped_local_param_ids = match KineticLaw::for_child_element(self.xml_element()) {
            Some(k) => k.local_parameter_identifiers(),
            None => Vec::new(),
        };

        let b_variables = self
            .recursive_child_elements()
            .into_iter()
            .filter(|child| child.tag_name() == "bvar")
            .filter_map(|bvar| bvar.get_child_at(0).map(|it| it.text_content()))
            .collect::<Vec<String>>();

        let ci_elements = self.recursive_child_elements_filtered(|child| child.tag_name() == "ci");

        for ci in ci_elements {
            let value = ci.text_content();
            if !b_variables.contains(&value)
                && all_local_param_ids.contains(&value)
                && !scoped_local_param_ids.contains(&value)
            {
                let message = format!("A <localParameter> identifier '{value}' found out of scope of its <KineticLaw>");
                issues.push(SbmlIssue::new_error("10216", &ci, message));
            }
        }
    }

    /// ### Rule 10218
    /// A MathML operator must be supplied the number of arguments appropriate for that operator.
    pub(crate) fn apply_rule_10218(&self, issues: &mut Vec<SbmlIssue>) {
        let apply_elements =
            self.recursive_child_elements_filtered(|child| child.tag_name() == "apply");

        for apply in apply_elements {
            let children = apply.child_elements();
            if children.is_empty() {
                let message = "No operator specified in <apply>.".to_string();
                issues.push(SbmlIssue::new_error("10218", &apply, message));
                continue;
            }
            let arg_count = children.len() - 1;
            let operator = children[0].tag_name();

            if MATHML_UNARY_OPERATORS.contains(&operator.as_str()) {
                // <minus> is allowed to have 1 OR 2 arguments
                if operator == "minus" && arg_count != 1 && arg_count != 2 {
                    let message = format!(
                        "Invalid number ({arg_count}) of arguments for operator <minus>. \
                                The operator <minus> can take either 1 or 2 arguments."
                    );
                    issues.push(SbmlIssue::new_error("10218", &apply, message));
                } else if operator != "minus" && arg_count != 1 {
                    let message = format!(
                        "Invalid number ({arg_count}) of arguments for unary operator <{operator}>"
                    );
                    issues.push(SbmlIssue::new_error("10218", &apply, message));
                }
            } else if MATHML_BINARY_OPERATORS.contains(&operator.as_str()) {
                // root is allowed to have 1 OR 2 arguments
                if operator == "root" && arg_count != 1 && arg_count != 2 {
                    let message = format!(
                        "Invalid number ({arg_count}) of arguments for operator <root>. \
                                The operator <root> can take either 1 or 2 arguments."
                    );
                    issues.push(SbmlIssue::new_error("10218", &apply, message));
                } else if operator != "root" && arg_count != 2 {
                    let message = format!("Invalid number ({arg_count}) of arguments for binary operator <{operator}>.");
                    issues.push(SbmlIssue::new_error("10218", &apply, message));
                }
            }
        }

        let piecewise_elements =
            self.recursive_child_elements_filtered(|child| child.tag_name() == "piecewise");

        for e in piecewise_elements {
            // Explicitly handle the piecewise operator that is technically n-ary, but must
            // have at least one child.
            let arg_count = e.child_elements().len();
            if arg_count < 1 {
                let message = format!(
                    "Invalid number ({arg_count}) of arguments for the operator <piecewise>. \
                        The operator <piecewise> must contain at least one <piece> or <otherwise> element."
                );
                issues.push(SbmlIssue::new_error("10218", &e, message));
            }
        }
    }

    /// ### Rule 10219
    /// The number of arguments used in a call to a function defined by a [FunctionDefinition] object must
    /// equal the number of arguments accepted by that function, if defined. In other words, it must equal
    /// the number of MathML **bvar** elements inside the **lambda** element of the function definition, if
    /// present.
    pub(crate) fn apply_rule_10219(&self, issues: &mut Vec<SbmlIssue>) {
        let model = Model::for_child_element(self.xml_element()).unwrap();

        let apply_elements =
            self.recursive_child_elements_filtered(|child| child.tag_name() == "apply");

        for apply in apply_elements {
            let children = apply.child_elements();
            let Some(function_call) = children.first() else {
                continue;
            };

            if function_call.tag_name() != "ci" {
                continue;
            }

            let arg_count = children.len() - 1;
            let func_identifiers = model.function_definition_identifiers();
            let id = function_call.text_content();

            if func_identifiers.contains(&id) {
                // Only check argument count if the function is actually declared.
                if let Some(expected_args) = model.function_definition_arguments(&id) {
                    if arg_count != expected_args {
                        let message = format!(
                            "Invalid number of arguments ({arg_count}) provided for function '{id}'. \
                                The function '{id}' takes {expected_args} arguments."
                        );
                        issues.push(SbmlIssue::new_error("10219", function_call, message));
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
        let children_of_interest =
            self.recursive_child_elements_filtered(|child| child.has_attribute("units"));

        for child in children_of_interest {
            let name = child.tag_name();

            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["units"].contains(&name.as_str()) {
                let message = format!(
                    "Attribute [units] found on element <{name}>, which is forbidden. \
                        Attribute [units] is only permitted on <cn>."
                );
                issues.push(SbmlIssue::new_error("10220", &child, message));
            }
        }
    }

    /// ### Rule 10221
    /// The value of the SBML attribute units on a MathML cn element must be chosen from either the
    /// set of identifiers of UnitDefinition objects in the model, or the set of base units defined by SBML.
    pub(crate) fn apply_rule_10221(&self, issues: &mut Vec<SbmlIssue>) {
        let unit_identifiers = Model::for_child_element(self.xml_element())
            .unwrap()
            .unit_definition_identifiers();
        let cn_elements = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "cn" && child.has_attribute("units")
        });

        for cn in cn_elements {
            // We can unwrap because we selected only elements where `units` attribute is set.
            let value = cn.get_attribute("units").unwrap();

            if !unit_identifiers.contains(&value) && BaseUnit::from_str(&value).is_err() {
                let message = format!(
                    "Invalid unit identifier '{value}' found. \
                        Only identifiers of <unitDefinition> objects and base units can be used in <cn>."
                );
                issues.push(SbmlIssue::new_error("10221", &cn, message));
            }
        }
    }

    /// ### Rule 10223
    /// The single argument for the *rateOf* **csymbol** function must be a **ci** element.
    pub(crate) fn apply_rule_10223(&self, issues: &mut Vec<SbmlIssue>) {
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "apply" && {
                if let Some(first_child) = child.get_child_at(0) {
                    first_child
                        .get_attribute("definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf")
                } else {
                    false
                }
            }
        });

        for child in children_of_interest {
            let apply_children = child.child_elements();

            if apply_children.len() != 2 {
                let message = format!(
                    "Invalid number ({}) of arguments provided for rateOf <csymbol>. \
                         The call of rateOf <csymbol> must have precisely one argument.",
                    apply_children.len() - 1
                );
                issues.push(SbmlIssue::new_error("10223", &child, message));
            } else {
                // This unwrap is ok because we only selected elements with at least one child.
                let argument_name = apply_children.last().unwrap().tag_name();

                if argument_name != "ci" {
                    let message = format!(
                        "Invalid argument <{argument_name}> provided for <csymbol>.\
                             The rateOf <csymbol> must have <ci> as its only argument."
                    );
                    issues.push(SbmlIssue::new_error("10223", &child, message));
                }
            }
        }
    }

    /// ### Rule 10224
    /// The target of a *rateOf* **csymbol** function must not appear as the *variable* of an
    /// [AssignmentRule](crate::core::rule::AssignmentRule), nor may its value be determined by an
    /// [AlgebraicRule](crate::core::rule::AlgebraicRule).
    pub(crate) fn apply_rule_10224(&self, issues: &mut Vec<SbmlIssue>) {
        let model = Model::for_child_element(self.xml_element()).unwrap();
        let ci_elements = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "apply" && {
                let children = child.child_elements();
                if children.len() < 2 {
                    false
                } else {
                    let fst = &children[0];
                    let snd = &children[1];
                    let is_rate_of = fst
                        .get_attribute("definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf");
                    let is_ci = snd.tag_name() == "ci";
                    is_ci && is_rate_of
                }
            }
        });
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_rule_parameters = model.algebraic_rule_ci_values();

        for ci in ci_elements {
            let value = ci.text_content();
            let is_target_constant = model.is_rateof_target_constant(value.as_str());

            if assignment_rule_variables.contains(&value) {
                let message = format!(
                    "The value of target ('{value}') of rateOf <csymbol> \
                found as a variable of <assignmentRule>."
                );
                issues.push(SbmlIssue::new_error("10224", &ci, message));
            } else if !is_target_constant && algebraic_rule_parameters.contains(&value) {
                let message = format!(
                    "The value of target ('{value}') of rateOf <csymbol> \
                determined by an <algebraicRule>."
                );
                issues.push(SbmlIssue::new_error("10224", &ci, message));
            }
        }
    }

    /// ### Rule 10225
    /// If the target of a *rateOf* **csymbol** function is a [Species](crate::core::species::Species) with a
    /// *hasOnlySubstanceUnits* value of *"false"*, the **compartment** of that [Species](crate::core::species::Species)
    /// must not appear as the *variable* of an [AssignmentRule](crate::core::rule::AssignmentRule),
    /// nor may its *size* be determined by an [AlgebraicRule](crate::core::rule::AlgebraicRule).
    pub(crate) fn apply_rule_10225(&self, issues: &mut Vec<SbmlIssue>) {
        let model = Model::for_child_element(self.xml_element()).unwrap();
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_ci_values = model.algebraic_rule_ci_values();
        let ci_elements = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "apply" && {
                let children = child.child_elements();
                if children.len() < 2 {
                    false
                } else {
                    let fst = &children[0];
                    let snd = &children[1];
                    let is_rate_of = fst
                        .get_attribute("definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf");
                    let is_ci = snd.tag_name() == "ci";
                    is_ci && is_rate_of
                }
            }
        });

        for ci in ci_elements {
            let value = ci.text_content();

            let Some(species) = model.find_species(value.as_str()) else {
                continue;
            };

            if species.has_only_substance_units().get() {
                continue;
            }

            let species_compartment = species.compartment().get();
            let Some(compartment) = model.find_compartment(species_compartment.as_str()) else {
                continue;
            };

            let compartment_id = compartment.id().get();

            if assignment_rule_variables.contains(&compartment_id) {
                let message = format!("The <compartment> with id '{compartment_id}' found as the [variable] of an <assignmentRule>.");
                issues.push(SbmlIssue::new_error("10225", &ci, message));
            } else if !compartment.constant().get() && algebraic_ci_values.contains(&compartment_id)
            {
                let message = format!("The <compartment>'s size with id '{compartment_id}' is possible to determine by an <algebraicRule>.");
                issues.push(SbmlIssue::new_error("10225", &ci, message));
            }
        }
    }

    /// ### Rule 10311
    /// The SBML *units* attribute on MathML **cn** elements must always conform to the syntax of the
    /// SBML data type **UnitSId**. Full description of the rule [here](crate::core::validation::apply_rule_10311).
    pub(crate) fn apply_rule_10311(&self, issues: &mut Vec<SbmlIssue>) {
        let cn_elements = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "cn" && child.has_attribute("units")
        });

        for cn in cn_elements {
            let value = cn.get_attribute("units");

            if !matches_unit_sid_pattern(&value) {
                let message = format!(
                    "The [units] value ('{0}') does not conform to the syntax of UnitSId data type.",
                    value.unwrap()
                );
                issues.push(SbmlIssue::new_error("10311", self.xml_element(), message))
            }
        }
    }

    /// ### Rule 10313
    /// The *units* attribute on MathML **ci** elements must be the identifier of a
    /// [UnitDefinition](crate::core::unit_definition::UnitDefinition) in the [Model], or the
    /// identifier of a predefined unit in SBML. Full description of the rule [here](apply_rule_10313);
    pub(crate) fn apply_rule_10313(&self, issues: &mut Vec<SbmlIssue>) {
        let ci_elements = self.recursive_child_elements_filtered(|child| {
            child.tag_name() == "ci" && child.has_attribute("units")
        });

        for ci in ci_elements {
            let value = ci.get_attribute("units");
            apply_rule_10313(ci.tag_name().as_str(), value, self.xml_element(), issues);
        }
    }
}
