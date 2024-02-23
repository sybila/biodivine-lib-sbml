use phf::phf_map;

macro_rules! extended_sbase_attributes {
    ($($y:expr),*) => {
        &["id", "name", "sboTerm", "metaid", $($y),*]
    };
}

macro_rules! extended_sbase_children {
    ($($y:expr),*) => {
        &["notes", "annotation", $($y),*]
    };
}
pub const ALLOWED_SBASE_ATTRIBUTES: &[&str] = extended_sbase_attributes!();
pub const ALLOWED_SBASE_CHILDREN: &[&str] = extended_sbase_children!();

pub const ALLOWED_ATTRIBUTES: phf::Map<&str, &[&str]> = phf_map! {
    "sbml" => extended_sbase_attributes!("xmlns", "level", "version"),
    "model"=> ALLOWED_SBASE_ATTRIBUTES,
    "listOfFunctionDefinitions" => ALLOWED_SBASE_ATTRIBUTES,
    "functionDefinition" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfUnitDefinitions" => ALLOWED_SBASE_ATTRIBUTES,
    "unitDefinition" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfUnits" => ALLOWED_SBASE_ATTRIBUTES,
    "unit" => extended_sbase_attributes!("kind", "exponent", "scale", "multiplier"),
    "listOfCompartments" => ALLOWED_SBASE_ATTRIBUTES,
    "compartment" => extended_sbase_attributes!("spatialDimensions", "size", "units", "constant"),
    "listOfSpecies" => ALLOWED_SBASE_ATTRIBUTES,
    "species" => extended_sbase_attributes!("compartment", "initialAmount", "initialConcentration", "substanceUnits", "hasOnlySubstanceUnits", "boundaryCondition", "constant", "conversionFactor"),
    "listOfParameters" => ALLOWED_SBASE_ATTRIBUTES,
    "parameter" => extended_sbase_attributes!("value", "units", "constant"),
    "listOfInitialAssignments" => ALLOWED_SBASE_ATTRIBUTES,
    "initialAssignment" => extended_sbase_attributes!("symbol"),
    "listOfRules" => ALLOWED_SBASE_ATTRIBUTES,
    "algebraicRule" => ALLOWED_SBASE_ATTRIBUTES,
    "assignmentRule" => extended_sbase_attributes!("variable"),
    "rateRule" => extended_sbase_attributes!("variable"),
    "listOfConstraints" => ALLOWED_SBASE_ATTRIBUTES,
    "constraint" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfReactions" => ALLOWED_SBASE_ATTRIBUTES,
    "reaction" => extended_sbase_attributes!("reversible", "compartment"),
    "listOfReactants" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfProducts" => ALLOWED_SBASE_ATTRIBUTES,
    "speciesReference" => extended_sbase_attributes!("species", "stoichiometry", "constant"),
    "listOfModifiers" => ALLOWED_SBASE_ATTRIBUTES,
    "modifierSpeciesReference" => extended_sbase_attributes!("species"),
    "kineticLaw" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfLocalParameters" => ALLOWED_SBASE_ATTRIBUTES,
    "localParameter" => extended_sbase_attributes!("value", "units"),
    "listOfEvents" => ALLOWED_SBASE_ATTRIBUTES,
    "event" => extended_sbase_attributes!("useValuesFromTriggerTime"),
    "trigger" => extended_sbase_attributes!("initialValue", "persistent"),
    "priority" => ALLOWED_SBASE_ATTRIBUTES,
    "delay" => ALLOWED_SBASE_ATTRIBUTES,
    "listOfEventAssignments" => ALLOWED_SBASE_ATTRIBUTES,
    "eventAssignment" => extended_sbase_attributes!("variable"),
};

pub const REQUIRED_ATTRIBUTES: phf::Map<&str, &[&str]> = phf_map! {
    "sbml" => &["level", "version"],
    "model" => &[],
    "listOfFunctionDefinitions" => &[],
    "functionDefinition" => &["id"],
    "listOfUnitDefinitions" => &[],
    "unitDefinition" => &["id"],
    "listOfUnits" => &[],
    "unit" => &["kind", "exponent", "scale", "multiplier"],
    "listOfCompartments" => &[],
    "compartment" => &["id", "constant"],
    "listOfSpecies" => &[],
    "species" => &["id", "compartment", "hasOnlySubstanceUnits", "boundaryCondition", "constant"],
    "listOfParameters" => &[],
    "parameter" => &["id", "constant"],
    "listOfInitialAssignments" => &[],
    "initialAssignment" => &["symbol"],
    "listOfRules" => &[],
    "algebraicRule" => &[],
    "assignmentRule" => &["variable"],
    "rateRule" => &["variable"],
    "listOfConstraints" => &[],
    "constraint" => &[],
    "listOfReactions" => &[],
    "reaction" => &["id", "reversible"],
    "listOfReactants" => &[],
    "listOfProducts" => &[],
    "speciesReference" => &["constant"],
    "listOfModifiers" => &[],
    "modifierSpeciesReference" => &[],
    "kineticLaw" => &[],
    "listOfLocalParameters" => &[],
    "localParameter" => &["id"],
    "listOfEvents" => &[],
    "event" => &["useValuesFromTriggerTime"],
    "trigger" => &["initialValue", "persistent"],
    "priority" => &[],
    "delay" => &[],
    "listOfEventAssignments" => &[],
    "eventAssignment" => &["variable"]
};

pub const ALLOWED_CHILDREN: phf::Map<&str, &[&str]> = phf_map! {
    "sbml" => extended_sbase_children!("model"),
    "model" => extended_sbase_children!("listOfFunctionDefinitions", "listOfUnitDefinitions", "listOfCompartments", "listOfSpecies", "listOfParameters", "listOfInitialAssignments", "listOfRules", "listOfConstraints", "listOfReactions", "listOfEvents"),
    "listOfFunctionDefinitions" => extended_sbase_children!("functionDefinition"),
    "functionDefinition" => extended_sbase_children!("math"),
    "listOfUnitDefinitions" => extended_sbase_children!("unitDefinition"),
    "unitDefinition" => extended_sbase_children!("listOfUnits"),
    "listOfUnits" => extended_sbase_children!("unit"),
    "unit" => extended_sbase_children!(),
    "listOfCompartments" => extended_sbase_children!("compartment"),
    "compartment" => extended_sbase_children!(),
    "listOfSpecies" => extended_sbase_children!("species"),
    "species" => extended_sbase_children!(),
    "listOfParameters" => extended_sbase_children!("parameter"),
    "parameter" => extended_sbase_children!(),
    "listOfInitialAssignments" => extended_sbase_children!("initialAssignment"),
    "initialAssignment" => extended_sbase_children!("math"),
    "listOfRules" => extended_sbase_children!("algebraicRule", "assignmentRule", "rateRule"),
    "algebraicRule" => extended_sbase_children!("math"),
    "assignmentRule" => extended_sbase_children!("math"),
    "rateRule" => extended_sbase_children!("math"),
    "listOfConstraints" => extended_sbase_children!("constraint"),
    "constraint" => extended_sbase_children!("math", "message"),
    "listOfReactions" => extended_sbase_children!("reaction"),
    "reaction" => extended_sbase_children!("listOfReactants", "listOfProducts", "listOfModifiers", "kineticLaw"),
    "listOfReactants" => extended_sbase_children!("speciesReference"),
    "listOfProducts" => extended_sbase_children!("speciesReference"),
    "speciesReference" => extended_sbase_children!(),
    "listOfModifiers" => extended_sbase_children!("modifierSpeciesReference"),
    "modifierSpeciesReference" => extended_sbase_children!(),
    "kineticLaw" => extended_sbase_children!("math", "listOfLocalParameters"),
    "listOfLocalParameters" => extended_sbase_children!("localParameter"),
    "localParameter" => extended_sbase_children!(),
    "listOfEvents" => extended_sbase_children!("event"),
    "event" => extended_sbase_children!("trigger", "priority", "delay", "listOfEventAssignments"),
    "trigger" => extended_sbase_children!("math"),
    "priority" => extended_sbase_children!("math"),
    "delay" => extended_sbase_children!("math"),
    "listOfEventAssignments" => extended_sbase_children!("eventAssignment"),
    "eventAssignment" => extended_sbase_children!("math")
};

// There are no required children in SBML core level 3 version 1

pub const MATHML_ALLOWED_CHILDREN: phf::Map<&str, &[&str]> = phf_map! {
    "math" => &["abs", "and", "annotation", "annotation-xml", "apply", "arccosh", "arccos", "arccoth",
                "arccot", "arccsch", "arccsc", "arcsech", "arcsec", "arcsinh", "arcsin", "arctanh",
                "arctan", "bvar", "ceiling", "ci", "cn", "cosh", "cos", "coth", "cot", "csch", "csc",
                "csymbol", "degree", "divide", "eq", "exponentiale", "exp", "factorial", "false",
                "floor", "geq", "gt", "implies", "infinity", "lambda", "leq", "ln", "logbase", "log",
                "lt", "max", "min", "minus", "neq", "notanumber", "not", "or", "otherwise", "piecewise",
                "piece", "pi", "plus", "power", "quotient", "rem", "root", "sech", "sec", "semantics",
                "sep", "sinh", "sin", "tanh", "tan", "times", "true", "xor"]
};

pub const MATHML_ALLOWED_CHILDREN_BY_ATTR: phf::Map<&str, &[&str]> = phf_map! {
    "encoding" => &["csymbol", "annotation", "annotation-xml"],
    "definitionURL" => &["ci", "csymbol", "semantics"],
    "type" => &["cn"],
    "units" => &["cn"]
};

pub const MATHML_ALLOWED_DEFINITION_URLS: &[&str] = &[
    "http://www.sbml.org/sbml/symbols/time",
    "http://www.sbml.org/sbml/symbols/delay",
    "http://www.sbml.org/sbml/symbols/avogadro",
    "http://www.sbml.org/sbml/symbols/rateOf",
];

pub const MATHML_ALLOWED_TYPES: &[&str] = &["e-notation", "real", "integer", "rational"];

// source: https://www.w3.org/TR/MathML2/chapter4.html#contm.funopqual
pub const MATHML_UNARY_OPERATORS: &[&str] = &[
    "factorial",
    "minus",
    "abs",
    "conjugate",
    "arg",
    "real",
    "imaginary",
    "floor",
    "ceiling",
    "not",
    "inverse",
    "ident",
    "domain",
    "codomain",
    "image",
    "sin",
    "cos",
    "tan",
    "sec",
    "csc",
    "cot",
    "sinh",
    "cosh",
    "tanh",
    "sech",
    "csch",
    "coth",
    "arcsin",
    "arccos",
    "arctan",
    "arccosh",
    "arccot",
    "arccoth",
    "arccsc",
    "arccsch",
    "arcsec",
    "arcsech",
    "arcsinh",
    "arctanh",
    "exp",
    "ln",
    "log",
    "determinant",
    "transpose",
    "divergence",
    "grad",
    "curl",
    "laplacian",
    "card",
];

// source: https://www.w3.org/TR/MathML2/chapter4.html#contm.funopqual
pub const MATHML_BINARY_OPERATORS: &[&str] = &[
    "quotient",
    "divide",
    "minus",
    "power",
    "rem",
    "root", // special operator of which one argument (degree) is by default 2 and therefore one argument is sufficient
    "implies",
    "equivalent",
    "approx",
    "setdiff",
    "vectorproduct",
    "scalarproduct",
    "outerproduct",
];

// source: https://www.w3.org/TR/MathML2/chapter4.html#contm.funopqual
pub const MATHML_NARY_OPERATORS: &[&str] = &[
    "plus",
    "times",
    "max",
    "min",
    "gcd",
    "lcm",
    "mean",
    "sdev",
    "variance",
    "median",
    "mode",
    "union",
    "intersect",
    "cartesianproduct",
    "selector",
    "and",
    "or",
    "xor",
    "eq",
    "neq",
    "leq",
    "lt",
    "geq",
    "gt",
];
