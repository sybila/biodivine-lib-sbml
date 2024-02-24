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
use crate::xml::{RequiredXmlProperty, XmlElement, XmlWrapper};
use crate::{SbmlIssue, SbmlIssueSeverity};

impl Math {
    /// Applies rules:
    ///  - **[10201](self.apply_rule_10201)** - MathML content is permitted only within [Math] element.
    ///  - **[10202](self.apply_rule_10202)** - Validates list of permitted elements within [Math] element.
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
            let message = format!(
                "Wrong namespace usage in a `math` element. Found `{}`, but `{}` should be used.",
                self.namespace_url(),
                URL_MATHML
            );
            issues.push(SbmlIssue::new_error("10201", self, message));
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10202
    /// Validates that only allowed subset of **MathML** child elements are present within [Math]
    /// element. An SBML package may allow new MathML elements to be added to this list, and if so,
    /// the package must define **required="true"** on the SBML container element
    /// [**sbml**](crate::Sbml).
    fn apply_rule_10202(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed_children = get_allowed_children(self.xml_element());

        for child in self.recursive_child_elements() {
            let child_tag_name = child.name(doc.deref());

            if !allowed_children.contains(&child_tag_name) {
                let message = format!(
                    "Unknown child <{0}> of element <{1}>.",
                    child_tag_name, "math"
                );
                issues.push(SbmlIssue::new_error("10202", self, message));
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
    fn apply_rule_10203(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["encoding"];
        let relevant_children = self.recursive_child_elements_filtered(|it| {
            it.attribute(doc.deref(), "encoding").is_some()
        });

        for child in relevant_children {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                let message = format!(
                    "Attribute [encoding] found on element <{0}>, which is forbidden. \
                        Attribute [encoding] is only permitted on <csymbol>, <annotation> and <annotation-xml>.",
                    name
                );
                issues.push(SbmlIssue::new_error("10203", self, message));
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
    fn apply_rule_10204(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let allowed = MATHML_ALLOWED_CHILDREN_BY_ATTR["definitionURL"];
        let relevant_children = self.recursive_child_elements_filtered(|it| {
            it.attribute(doc.deref(), "definitionURL").is_some()
        });

        for child in relevant_children {
            let name = child.name(doc.deref());

            if !allowed.contains(&name) {
                let message = format!(
                    "Attribute [definitionURL] found on element <{0}>, which is forbidden. \
                        Attribute [definitionURL] is only permitted on <ci>, <csymbol> and <semantics>.",
                    name
                );

                issues.push(SbmlIssue::new_error("10204", self, message));
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
    fn apply_rule_10205(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.attribute(doc.deref(), "definitionURL").is_some()
                && child.name(doc.deref()) == "csymbol"
        });

        for child in children_of_interest {
            let value = child.attribute(doc.deref(), "definitionURL").unwrap();
            if !MATHML_ALLOWED_DEFINITION_URLS.contains(&value) {
                let message = format!(
                    "Invalid definitionURL value found '{}'. Permitted values are: {:?}",
                    value, MATHML_ALLOWED_DEFINITION_URLS
                );
                issues.push(SbmlIssue::new_error("10205", self, message));
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
    fn apply_rule_10206(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.attribute(doc.deref(), "type").is_some()
        });

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["type"].contains(&name) {
                let message = format!(
                    "Attribute [type] found on element <{0}>, which is forbidden. \
                        Attribute [type] is only permitted on <cn>.",
                    name
                );
                issues.push(SbmlIssue::new_error("10204", self, message));
            }
        }
    }

    // TODO: Complete implementation when adding extensions/packages is solved
    /// ### Rule 10207
    /// The only permitted values for the attribute **type** on MathML cn elements are
    /// "**e-notation**", "**real**", "**integer**", and "**rational**". An SBML package may
    /// allow new values for the type attribute, and if so, the package must define
    /// **required="true"** on the SBML container element [**sbml**](crate::Sbml).
    fn apply_rule_10207(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.attribute(doc.deref(), "type").is_some()
        });

        for child in children_of_interest {
            let value = child.attribute(doc.deref(), "type").unwrap();

            if !MATHML_ALLOWED_TYPES.contains(&value) {
                let message = format!(
                    "Invalid type value found '{0}'. Permitted values are: \
                    'e-notation', 'real', 'integer' and 'rational'",
                    value
                );
                issues.push(SbmlIssue::new_error("10206", self, message));
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
    fn apply_rule_10208(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest =
            self.recursive_child_elements_filtered(|child| child.name(doc.deref()) == "lambda");

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
                let message = format!(
                    "Invalid immediate parent of <lambda>. Only <math> and <semantics> are \
                        valid immediate parents. Actual parent: <{0}>",
                    parent_name
                );
                issues.push(SbmlIssue::new_error("10208", self, message));
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
    /// Outside a [FunctionDefinition] object, if a MathML
    /// **ci** element is the first element within a MathML apply element, then the **ci** element's
    /// value can only be chosen from the set of identifiers of
    /// [FunctionDefinition] objects defined in the enclosing
    /// SBML [Model] object.
    fn apply_rule_10214(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let parent_name = self
            .raw_element()
            .parent(doc.deref())
            .unwrap()
            .name(doc.deref());

        if parent_name != "functionDefinition" {
            let children_of_interest = self.recursive_child_elements_filtered(|child| {
                let is_apply = child.name(doc.deref()) == "apply";
                let ci_first = child
                    .child_elements(doc.deref())
                    .first()
                    .map(|it| it.name(doc.deref()) == "ci")
                    .unwrap_or(false);
                is_apply && ci_first
            });

            let identifiers = Model::for_child_element(self.document(), self.xml_element())
                .unwrap()
                .function_definition_identifiers();

            for child in children_of_interest {
                // This unwrap must succeed because we enforced that ci is the first child.
                let value = child
                    .child_elements(doc.deref())
                    .first()
                    .unwrap()
                    .text_content(doc.deref());

                if !identifiers.contains(&value) {
                    let message = format!(
                        "Function '{0}' not defined. \
                            Function referred by <ci> must be defined in <functionDefinition> object \
                            with relevant identifier (id).",
                        value
                    );
                    issues.push(SbmlIssue::new_error("10214", self, message));
                }
            }
        }
    }

    // TODO: needs review
    /// ### Rule 10215
    /// Outside a [FunctionDefinition] object, if a MathML **ci** element is not the first element within
    /// a MathML **apply**, then the **ci** element's value may only be chosen from the following set of
    /// identifiers: the identifiers of [Species], [Compartment], [Parameter], [SpeciesReference]
    /// and [Reaction] objects defined in the enclosing [Model] object; the identifiers of
    /// [LocalParameter] objects that are children of the [Reaction] in which the
    /// [FunctionDefinition] appears (if it appears inside the [Math] object of a [KineticLaw]);
    /// and any identifiers (in the SId namespace of the model) belonging to an
    /// object class defined by an SBML Level 3 package as having mathematical meaning.
    fn apply_rule_10215(&self, issues: &mut Vec<SbmlIssue>) {
        let is_out_of_function_definition =
            FunctionDefinition::for_child_element(self.document(), self.xml_element()).is_none();

        if !is_out_of_function_definition {
            return;
        }

        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
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
            self.recursive_child_elements_filtered(|child| child.name(doc.deref()) == "apply");

        for apply in apply_elements {
            let ci_elements = apply
                .child_elements(doc.deref())
                .into_iter()
                .skip(1)
                .filter(|child| child.name(doc.deref()) == "ci")
                .collect::<Vec<_>>();

            for ci in ci_elements {
                let value = ci.text_content(doc.deref());

                if !identifiers.contains(&value) {
                    let ci = XmlElement::new_raw(self.document(), ci);
                    let message = format!(
                        "Invalid identifier value '{0}' in <ci>. Identifier not found.",
                        value
                    );
                    issues.push(SbmlIssue::new_error("10215", &ci, message));
                }
            }
        }
    }

    // TODO: needs review
    /// ### Rule 10216
    /// The id attribute value of a [LocalParameter] object defined within a [KineticLaw] object may only be
    /// used, in core, in MathML ci elements within the math element of that same [KineticLaw]; in other
    /// words, the identifier of the [LocalParameter] object is not visible to other parts of the model outside
    /// of that [Reaction] instance. In package constructs, the **id** attribute value of a [LocalParameter] object
    /// may only be used in MathML ci elements or as the target of an SIdRef attribute if that package
    /// construct is a child of the parent [Reaction].
    fn apply_rule_10216(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let all_local_param_ids = model.local_parameter_identifiers();
        let scoped_local_param_ids =
            match KineticLaw::for_child_element(self.document(), self.xml_element()) {
                Some(k) => k.local_parameter_identifiers(),
                None => Vec::new(),
            };
        let b_variables = self
            .recursive_child_elements()
            .into_iter()
            .filter(|child| child.name(doc.deref()) == "bvar")
            .filter_map(|bvar| {
                bvar.child_elements(doc.deref())
                    .first()
                    .map(|it| it.text_content(doc.deref()))
            })
            .collect::<Vec<String>>();

        let ci_elements =
            self.recursive_child_elements_filtered(|child| child.name(doc.deref()) == "ci");

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());
            if !b_variables.contains(&value)
                && all_local_param_ids.contains(&value)
                && !scoped_local_param_ids.contains(&value)
            {
                let ci = XmlElement::new_raw(self.document(), ci);
                let message = format!(
                    "A <localParameter> identifier '{0}' found out of scope of its <KineticLaw>",
                    value
                );
                issues.push(SbmlIssue::new_error("10216", &ci, message));
            }
        }
    }

    /// ### Rule 10218
    /// A MathML operator must be supplied the number of arguments appropriate for that operator.
    fn apply_rule_10218(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let apply_elements =
            self.recursive_child_elements_filtered(|child| child.name(doc.deref()) == "apply");

        for apply in apply_elements {
            let apply = XmlElement::new_raw(self.document(), apply);
            let children = apply.child_elements();
            let child_count = children.len();
            if child_count == 0 {
                let message = "No operator specified in <apply>.".to_string();
                issues.push(SbmlIssue::new_error("10218", &apply, message));
                continue;
            }
            let arg_count = child_count - 1;
            let operator = children[0].name(doc.deref());

            if MATHML_UNARY_OPERATORS.contains(&operator) {
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
            } else if MATHML_BINARY_OPERATORS.contains(&operator) {
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
            } else if MATHML_NARY_OPERATORS.contains(&operator) && arg_count == 0 {
                // TODO:
                //  This is not correct? N-ary operators with zero arguments are only
                //  discouraged if the meaning of the operator is not well defined.
                let message = format!("An N-ary operator <{operator}> with 0 arguments found. Use of N-ary operators without any arguments is discouraged.");
                issues.push(SbmlIssue {
                    element: apply.raw_element(),
                    message,
                    rule: "10218".to_string(),
                    severity: SbmlIssueSeverity::Warning,
                });
            }
        }
    }

    /// ### Rule 10219
    fn apply_rule_10219(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();

        let apply_elements =
            self.recursive_child_elements_filtered(|child| child.name(doc.deref()) == "apply");

        for apply in apply_elements {
            let children = apply.child_elements(doc.deref());
            let Some(function_call) = children.first() else {
                continue;
            };

            if function_call.name(doc.deref()) != "ci" {
                continue;
            }

            let arg_count = children.len() - 1;
            let func_identifiers = model.function_definition_identifiers();
            let id = function_call.text_content(doc.deref());

            if func_identifiers.contains(&id) {
                let expected_args = model
                    .function_definition_arguments(id.as_str())
                    .unwrap_or(0);

                if arg_count != expected_args {
                    let message = format!(
                        "Invalid number of arguments ({arg_count}) provided for function '{id}'. \
                                The function '{id}' takes {expected_args} arguments."
                    );
                    let function_call = XmlElement::new_raw(self.document(), *function_call);
                    issues.push(SbmlIssue::new_error("10219", &function_call, message));
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
    fn apply_rule_10220(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest: Vec<Element> = self.recursive_child_elements_filtered(|child| {
            child.attribute(doc.deref(), "units").is_some()
        });

        for child in children_of_interest {
            let name = child.name(doc.deref());

            if !MATHML_ALLOWED_CHILDREN_BY_ATTR["units"].contains(&name) {
                let message = format!(
                    "Attribute [units] found on element <{name}>, which is forbidden. \
                        Attribute [units] is only permitted on <cn>."
                );
                let child = XmlElement::new_raw(self.document(), child);
                issues.push(SbmlIssue::new_error("10220", &child, message));
            }
        }
    }

    /// ### Rule 10221
    /// The value of the SBML attribute units on a MathML cn element must be chosen from either the
    /// set of identifiers of UnitDefinition objects in the model, or the set of base units defined by SBML.
    fn apply_rule_10221(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let unit_identifiers = Model::for_child_element(self.document(), self.xml_element())
            .unwrap()
            .unit_definition_identifiers();
        let cn_elements = self.recursive_child_elements_filtered(|child| {
            child.name(doc.deref()) == "cn" && child.attribute(doc.deref(), "units").is_some()
        });

        for cn in cn_elements {
            let value = cn.attribute(doc.deref(), "units").unwrap();

            if !unit_identifiers.contains(&value.to_string()) && BaseUnit::from_str(value).is_err()
            {
                let message = format!(
                    "Invalid unit identifier '{value}' found. \
                        Only identifiers of <unitDefinition> objects and base units can be used in <cn>."
                );
                let cn = XmlElement::new_raw(self.document(), cn);
                issues.push(SbmlIssue::new_error("10221", &cn, message));
            }
        }
    }

    /// ### Rule 10223
    /// The single argument for the *rateOf* **csymbol** function must be a **ci** element.
    fn apply_rule_10223(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let children_of_interest = self.recursive_child_elements_filtered(|child| {
            child.name(doc.deref()) == "apply" && {
                if let Some(first_child) = child.child_elements(doc.deref()).first() {
                    first_child
                        .attribute(doc.deref(), "definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf")
                } else {
                    false
                }
            }
        });

        for child in children_of_interest {
            let apply_children = child.child_elements(doc.deref());

            if apply_children.len() != 2 {
                let message = format!(
                    "Invalid number ({0}) of arguments provided for rateOf <csymbol>. \
                         The call of rateOf <csymbol> must have precisely one argument.",
                    apply_children.len() - 1
                );
                let child = XmlElement::new_raw(self.document(), child);
                issues.push(SbmlIssue::new_error("10223", &child, message));
            } else {
                // This unwrap is ok because we only selected elements with at least one child.
                let argument_name = apply_children.last().unwrap().name(doc.deref());

                if argument_name != "ci" {
                    let message = format!(
                        "Invalid argument <{0}> provided for <csymbol>.\
                             The rateOf <csymbol> must have <ci> as its only argument.",
                        argument_name
                    );
                    let child = XmlElement::new_raw(self.document(), child);
                    issues.push(SbmlIssue::new_error("10223", &child, message));
                }
            }
        }
    }

    /// ### Rule 10224
    /// The target of a *rateOf* **csymbol** function must not appear as the *variable* of an
    /// [AssignmentRule](crate::core::rule::AssignmentRule), nor may its value be determined by an
    /// [AlgebraicRule](crate::core::rule::AlgebraicRule).
    fn apply_rule_10224(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let ci_elements = self.recursive_child_elements_filtered(|child| {
            child.name(doc.deref()) == "apply" && {
                let children = child.child_elements(doc.deref());
                if children.len() < 2 {
                    false
                } else {
                    let fst = children[0];
                    let snd = children[1];
                    let is_rate_of = fst
                        .attribute(doc.deref(), "definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf");
                    let is_ci = snd.name(doc.deref()) == "ci";
                    is_ci && is_rate_of
                }
            }
        });
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_rule_determinants = model.algebraic_rule_ci_values();

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());

            if assignment_rule_variables.contains(&value) {
                let message = format!("The value of target ('{value}') of rateOf <csymbol> found as a variable of <assignmentRule>.");
                let ci = XmlElement::new_raw(self.document(), ci);
                issues.push(SbmlIssue::new_error("10224", &ci, message));
                // TODO: what does "determined by algebraicRule" mean and how to check it?
            } else if algebraic_rule_determinants.contains(&value) {
                let message = format!("The value of target ('{value}') of rateOf <csymbol> determined by an <algebraicRule>.");
                let ci = XmlElement::new_raw(self.document(), ci);
                issues.push(SbmlIssue::new_error("10224", &ci, message));
            }
        }
    }

    /// ### Rule 10225
    /// If the target of a *rateOf* **csymbol** function is a [Species](crate::core::species::Species) with a
    /// *hasOnlySubstanceUnits* value of *"false"*, the **compartment** of that [Species](crate::core::species::Species)
    /// must not appear as the *variable* of an [AssignmentRule](crate::core::rule::AssignmentRule),
    /// nor may its *size* be determined by an [AlgebraicRule](crate::core::rule::AlgebraicRule).
    fn apply_rule_10225(&self, issues: &mut Vec<SbmlIssue>) {
        let doc = self.read_doc();
        let model = Model::for_child_element(self.document(), self.xml_element()).unwrap();
        let assignment_rule_variables = model.assignment_rule_variables();
        let algebraic_ci_values = model.algebraic_rule_ci_values();
        let ci_elements = self.recursive_child_elements_filtered(|child| {
            child.name(doc.deref()) == "apply" && {
                let children = child.child_elements(doc.deref());
                if children.len() < 2 {
                    false
                } else {
                    let fst = children[0];
                    let snd = children[1];
                    let is_rate_of = fst
                        .attribute(doc.deref(), "definitionURL")
                        .is_some_and(|url| url == "http://www.sbml.org/sbml/symbols/rateOf");
                    let is_ci = snd.name(doc.deref()) == "ci";
                    is_ci && is_rate_of
                }
            }
        });

        for ci in ci_elements {
            let value = ci.text_content(doc.deref());

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
                let ci = XmlElement::new_raw(self.document(), ci);
                issues.push(SbmlIssue::new_error("10225", &ci, message));
            } else if !compartment.constant().get() && algebraic_ci_values.contains(&compartment_id)
            {
                let message = format!("The <compartment>'s size with id '{compartment_id}' is possible to determine by an <algebraicRule>.");
                let ci = XmlElement::new_raw(self.document(), ci);
                issues.push(SbmlIssue::new_error("10225", &ci, message));
            }
        }
    }
}
