// TODO:
//      Check that escaping rules are obeyed for a "generic" string type (see specification
//      section 3.1.1). I believe these should be handled by `xml-doc` already, but we should
//      have a test case for this.
use crate::xml::XmlPropertyType;
use sbml_macros::make_python_property;

// Implementations of Python property converters for native types:
make_python_property!(String);
make_python_property!(bool);
make_python_property!(i32);
make_python_property!(u32);
make_python_property!(f64);

/// A "trivial" conversion between an XML attribute and a `String`.
///
/// ## Specification
///  - Section 3.1.1
impl XmlPropertyType for String {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        Ok(value.map(|it| it.to_string()))
    }

    fn set(&self) -> Option<String> {
        Some(self.clone())
    }
}

/// A "trivial conversion between an XML attribute and a `bool`.
///
/// Note that (per specification), both `0/1` and `true/false` are allowed here. However, when
/// writing, `true/false` notation is preferred. This ensures that the output is compatible with
/// both SBML and MathML.
///
/// ## Specification
///  - Section 3.1.2
impl XmlPropertyType for bool {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some("1") | Some("true") => Ok(Some(true)),
            Some("0") | Some("false") => Ok(Some(false)),
            Some(value) => Err(format!(
                "Value '{value}' does not represent a valid 'bool'."
            )),
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(if *self { "true" } else { "false" }.to_string())
    }
}

/// A "trivial" conversion between an XML attribute and a `i32` integer (`int` type in the SBML
/// specification).
///
/// As far as I know, the default algorithm for parsing/printing integers should be equivalent
/// to the representation expected by SBML.
///
/// ## Specification
///  - Section 3.1.3
impl XmlPropertyType for i32 {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        if let Some(value) = value {
            match value.parse::<i32>() {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(format!(
                    "Value '{value}' does not represent a valid signed integer ({}).",
                    e
                )),
            }
        } else {
            Ok(None)
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl XmlPropertyType for u32 {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        if let Some(value) = value {
            match value.parse::<u32>() {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(format!(
                    "Value '{value}' does not represent a valid unsigned integer ({}).",
                    e
                )),
            }
        } else {
            Ok(None)
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

/// A "trivial" conversion between an XML attribute and a `f64` floating-point number (`double`
/// type in the SBML specification). Missing attribute value is interpreted as an error.
///
/// ## Specification
///  - Section 3.1.5
impl XmlPropertyType for f64 {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match value.parse::<f64>() {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(format!(
                    "Value '{value}' does not represent a valid floating point number ({}).",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}
