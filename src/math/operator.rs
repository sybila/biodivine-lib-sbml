//! The closed operator and constant sets of the SBML MathML subset.
//!
//! MathML operators and constants are *not* SBML objects of their own: they are empty elements of
//! the MathML namespace whose meaning is entirely determined by their element name (`<plus/>`,
//! `<pi/>`), and the SBML specification fixes exactly which of them are permitted (see the module
//! documentation of [`crate::math`]). They are therefore plain enumerations with an
//! element-name table, not declared wrapper types — the identity machinery of `crate::xml` is for
//! elements with a structure, and adding 58 identity rows for elements that have none would only
//! blur what a declared identity means.
//!
//! The three enumerations are *closed*: [`MathOperator::from_name`] answers `None` for a name
//! outside the subset (for example `gcd`, which MathML 2.0 defines but SBML does not allow), which
//! is what [`MathNode::try_from_element`](crate::math::MathNode::try_from_element) reports as a
//! type verdict. The element of such a name stays reachable through the raw
//! [`XmlElement`](crate::xml::XmlElement).

use crate::constants::element::{MATHML_BINARY_OPERATORS, MATHML_UNARY_OPERATORS};
use std::fmt::{Display, Formatter};

/// Declares an enumeration whose variants are the elements of a closed MathML name set.
///
/// Every variant carries its element name exactly once, and the macro derives the whole table from
/// that: the `ALL` list (which the tests pin against the validation constants), the
/// name -> element name and element name -> variant directions, and the `Display` implementation.
macro_rules! math_name_set {
    ($(#[$outer:meta])* $name:ident { $($(#[$inner:meta])* $variant:ident => $element:literal,)* }) => {
        $(#[$outer])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$inner])* $variant,)*
        }

        impl $name {
            /// Every value of this enumeration, in declaration order.
            pub const ALL: &'static [Self] = &[$($name::$variant,)*];

            /// The local name of the MathML element this value stands for.
            pub const fn name(self) -> &'static str {
                match self {
                    $($name::$variant => $element,)*
                }
            }

            /// The value whose element has this local name, or `None` when the name is outside the
            /// subset.
            ///
            /// The lookup is by *local* name; whether the element is in the MathML namespace is
            /// decided by the caller (see [`MathNode::try_from_element`](crate::math::MathNode::try_from_element)).
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($element => Some($name::$variant),)*
                    _ => None,
                }
            }
        }

        impl Display for $name {
            /// Writes the element name, which is how the value is spelled in a document.
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}

math_name_set! {
    /// A MathML operator that SBML permits inside an `apply` element.
    ///
    /// The set is the 52 operator names of the permitted subset; the argument count rule 10218
    /// accepts for each of them is [`MathOperator::arity`]. The value is a *name*, not a language
    /// construct: `<minus/>` with one argument is negation and with two is subtraction, and the
    /// freeze of the meaning belongs to MathML 2.0.
    MathOperator {
        /// `<abs/>`
        Abs => "abs",
        /// `<and/>`
        And => "and",
        /// `<arccosh/>`
        Arccosh => "arccosh",
        /// `<arccos/>`
        Arccos => "arccos",
        /// `<arccoth/>`
        Arccoth => "arccoth",
        /// `<arccot/>`
        Arccot => "arccot",
        /// `<arccsch/>`
        Arccsch => "arccsch",
        /// `<arccsc/>`
        Arccsc => "arccsc",
        /// `<arcsech/>`
        Arcsech => "arcsech",
        /// `<arcsec/>`
        Arcsec => "arcsec",
        /// `<arcsinh/>`
        Arcsinh => "arcsinh",
        /// `<arcsin/>`
        Arcsin => "arcsin",
        /// `<arctanh/>`
        Arctanh => "arctanh",
        /// `<arctan/>`
        Arctan => "arctan",
        /// `<ceiling/>`
        Ceiling => "ceiling",
        /// `<cosh/>`
        Cosh => "cosh",
        /// `<cos/>`
        Cos => "cos",
        /// `<coth/>`
        Coth => "coth",
        /// `<cot/>`
        Cot => "cot",
        /// `<csch/>`
        Csch => "csch",
        /// `<csc/>`
        Csc => "csc",
        /// `<divide/>`
        Divide => "divide",
        /// `<eq/>`
        Eq => "eq",
        /// `<exp/>`
        Exp => "exp",
        /// `<factorial/>`
        Factorial => "factorial",
        /// `<floor/>`
        Floor => "floor",
        /// `<geq/>`
        Geq => "geq",
        /// `<gt/>`
        Gt => "gt",
        /// `<implies/>`
        Implies => "implies",
        /// `<leq/>`
        Leq => "leq",
        /// `<ln/>`
        Ln => "ln",
        /// `<log/>`
        Log => "log",
        /// `<lt/>`
        Lt => "lt",
        /// `<max/>`
        Max => "max",
        /// `<min/>`
        Min => "min",
        /// `<minus/>`
        Minus => "minus",
        /// `<neq/>`
        Neq => "neq",
        /// `<not/>`
        Not => "not",
        /// `<or/>`
        Or => "or",
        /// `<plus/>`
        Plus => "plus",
        /// `<power/>`
        Power => "power",
        /// `<quotient/>`
        Quotient => "quotient",
        /// `<rem/>`
        Rem => "rem",
        /// `<root/>`
        Root => "root",
        /// `<sech/>`
        Sech => "sech",
        /// `<sec/>`
        Sec => "sec",
        /// `<sinh/>`
        Sinh => "sinh",
        /// `<sin/>`
        Sin => "sin",
        /// `<tanh/>`
        Tanh => "tanh",
        /// `<tan/>`
        Tan => "tan",
        /// `<times/>`
        Times => "times",
        /// `<xor/>`
        Xor => "xor",
    }
}

math_name_set! {
    /// A MathML constant that SBML permits as a leaf of a formula.
    ///
    /// These are the six constant elements of the subset (prose Section 3.4.1). `true` and `false`
    /// are the Boolean constants; the remaining four are the numeric ones.
    MathConstant {
        /// `<true/>`
        True => "true",
        /// `<false/>`
        False => "false",
        /// `<notanumber/>`
        NotANumber => "notanumber",
        /// `<pi/>`
        Pi => "pi",
        /// `<infinity/>`
        Infinity => "infinity",
        /// `<exponentiale/>`
        ExponentialE => "exponentiale",
    }
}

/// The number of arguments rule 10218 accepts for an operator of the subset.
///
/// This is *information*, not enforcement: the builders build whatever the caller asks for, because
/// the crate has to be able to edit a document that breaks a rule. `arity` mirrors what the rule
/// checks and is pinned against the validation constants by a test, so a change to the rule becomes
/// visible here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathArity {
    /// Exactly one argument: `abs`, `sin`, `factorial`, …
    Unary,
    /// Exactly two arguments: `divide`, `power`, `implies`, …
    Binary,
    /// One or two arguments: `minus` (negation or subtraction) and `root` (whose `degree` is
    /// optional). These are the two operators rule 10218 special-cases by name.
    UnaryOrBinary,
    /// Any number of arguments, including none: `plus`, `times`, `and`, `or`, … — the operators
    /// whose arguments rule 10218 does not count.
    Nary,
}

/// The operators whose argument count rule 10218 accepts to be one *or* two.
///
/// `minus` is in both the unary and the binary table of the specification, and `root` is the one
/// binary operator with an optional `degree` argument, which the rule reads as an argument of its
/// own (so a two-argument `<root>` call is legal). The list is the rule's exception set written
/// down as data.
const UNARY_OR_BINARY_OPERATORS: &[&str] = &["minus", "root"];

impl MathOperator {
    /// The argument count rule 10218 accepts for this operator.
    ///
    /// The answer is derived from the specification's operator tables
    /// (`MATHML_UNARY_OPERATORS`/`MATHML_BINARY_OPERATORS`) plus the two exceptions above, so it
    /// cannot drift from what the validator checks without a test failing.
    pub fn arity(self) -> MathArity {
        let name = self.name();
        if UNARY_OR_BINARY_OPERATORS.contains(&name) {
            // `minus` (unary and binary) and `root` (optional `degree`).
            return MathArity::UnaryOrBinary;
        }
        match (
            MATHML_UNARY_OPERATORS.contains(&name),
            MATHML_BINARY_OPERATORS.contains(&name),
        ) {
            (true, false) => MathArity::Unary,
            (false, true) => MathArity::Binary,
            // Neither table names these: the N-ary operators, whose argument count the rule does
            // not constrain.
            (false, false) => MathArity::Nary,
            // Unreachable while `minus` is the only name in both tables; answered as the exception
            // list does, so the meaning stays "rule 10218 accepts either count".
            (true, true) => MathArity::UnaryOrBinary,
        }
    }
}

/// One of the four symbols of the SBML namespace that a `csymbol` element can denote.
///
/// In SBML Level 3, a `csymbol` element whose `definitionURL` is one of these four URLs denotes
/// something that the model itself defines: the current simulation time, an event delay, the
/// Avogadro constant, or the rate of change of a species. Rules 10205, 10223, 10224 and 10225 are
/// all about them. A `csymbol` that denotes something else (a symbol of an SBML package, for
/// example) is built with a `definitionURL` of its own; see
/// [`MathNode::csymbol_with_url`](crate::math::MathNode::csymbol_with_url).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathSymbol {
    /// `http://www.sbml.org/sbml/symbols/time` — the current time of the simulation.
    Time,
    /// `http://www.sbml.org/sbml/symbols/delay` — the delay of an event.
    Delay,
    /// `http://www.sbml.org/sbml/symbols/avogadro` — the Avogadro constant.
    Avogadro,
    /// `http://www.sbml.org/sbml/symbols/rateOf` — the rate of change of a species.
    RateOf,
}

impl MathSymbol {
    /// Every value of this enumeration, in declaration order.
    pub const ALL: &'static [Self] = &[
        MathSymbol::Time,
        MathSymbol::Delay,
        MathSymbol::Avogadro,
        MathSymbol::RateOf,
    ];

    /// The value of the `definitionURL` attribute that denotes this symbol.
    pub const fn definition_url(self) -> &'static str {
        match self {
            MathSymbol::Time => "http://www.sbml.org/sbml/symbols/time",
            MathSymbol::Delay => "http://www.sbml.org/sbml/symbols/delay",
            MathSymbol::Avogadro => "http://www.sbml.org/sbml/symbols/avogadro",
            MathSymbol::RateOf => "http://www.sbml.org/sbml/symbols/rateOf",
        }
    }

    /// The symbol whose `definitionURL` is this URL, if it is one of the four SBML symbols.
    pub fn from_url(url: &str) -> Option<Self> {
        // The table is derived from the constant the validator uses (rule 10205), so the two
        // cannot disagree about which URLs are the SBML symbols.
        MathSymbol::ALL
            .iter()
            .copied()
            .find(|it| it.definition_url() == url)
    }
}

impl Display for MathSymbol {
    /// Writes the `definitionURL` of the symbol.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.definition_url())
    }
}
