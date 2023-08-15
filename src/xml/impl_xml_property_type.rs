// TODO:
//      Check that escaping rules are obeyed for a "generic" string type (see specification
//      section 3.1.1). I believe these should be handled by `xml-doc` already, but we should
//      have a test case for this.

use std::str::FromStr;

use crate::{model::BaseUnit, xml::XmlPropertyType};

/// A "trivial" conversion between an XML attribute and a `String`. When the attribute value
/// is missing, returns a "Value missing." error.
///
/// ## Specification
///  - Section 3.1.1
impl XmlPropertyType for String {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        value
            .map(|it| it.to_string())
            .ok_or("Value missing.".to_string())
    }

    fn write(&self) -> Option<String> {
        Some(self.clone())
    }
}

/// A "trivial" conversion between an XML attribute and an optional `String`. The `None` value
/// maps to a missing XML attribute.
///
/// ## Specification
///  - Section 3.1.1
impl XmlPropertyType for Option<String> {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        Ok(value.map(|it| it.to_string()))
    }

    fn write(&self) -> Option<String> {
        self.clone()
    }
}

/// A "trivial conversion between an XML attribute and a `bool`. Missing attribute value is
/// interpreted as an error.
///
/// Note that (per specification), both `0/1` and `true/false` are allowed here. However, when
/// writing, `true/false` notation is preferred. This ensures that the output is compatible with
/// both SBML and MathML.
///
/// ## Specification
///  - Section 3.1.2
impl XmlPropertyType for bool {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        match value {
            None => Err("Value missing.".to_string()),
            Some("1") | Some("true") => Ok(true),
            Some("0") | Some("false") => Ok(false),
            Some(value) => Err(format!(
                "Value `{value}` does not represent a valid `bool`."
            )),
        }
    }

    fn write(&self) -> Option<String> {
        Some(if *self { "true" } else { "false" }.to_string())
    }
}

/// A "trivial" conversion between an XML attribute and a `i32` integer (`int` type in the SBML
/// specification). Missing attribute value is interpreted as an error.
///
/// As far as I know, the default algorithm for parsing/printing integers should be equivalent
/// to the representation expected by SBML.
///
/// ## Specification
///  - Section 3.1.3
impl XmlPropertyType for i32 {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        if let Some(value) = value {
            match value.parse::<i32>() {
                Ok(x) => Ok(x),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid signed integer ({}).",
                    e
                )),
            }
        } else {
            Err("Value missing".to_string())
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

/// A "trivial" conversion between an XML attribute and a `f64` floating-point number (`double`
/// type in the SBML specification). Missing attribute value is interpreted as an error.
///
/// ## Specification
///  - Section 3.1.5
impl XmlPropertyType for f64 {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(value) => match value.parse::<f64>() {
                Ok(x) => Ok(x),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid signed integer ({}).",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

/// A "trivial" conversion between an XML attribute and a `f64` floating-point number (`double`
/// type in the SBML specification).
///
/// ## Specification
///  - Section 3.1.5
impl XmlPropertyType for Option<f64> {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(_) => Ok(Some(f64::try_read(value).unwrap())),
            None => Ok(None),
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{:#?}", self))
    }
}

/// A conversion between an XML attribute and a [BaseUnit] value. Missing attribute value is
/// interpreted as an error.
///
/// ## Specification
///  - Section 4.4.2
impl XmlPropertyType for BaseUnit {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(value) => match BaseUnit::from_str(value) {
                Ok(unit) => Ok(unit),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid base unit ({})",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl XmlPropertyType for Option<BaseUnit> {
    fn try_read(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(_) => Ok(Some(BaseUnit::try_read(value).unwrap())),
            None => Ok(None),
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{:#?}", self))
    }
}
