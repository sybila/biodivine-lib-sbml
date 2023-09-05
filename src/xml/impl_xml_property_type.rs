// TODO:
//      Check that escaping rules are obeyed for a "generic" string type (see specification
//      section 3.1.1). I believe these should be handled by `xml-doc` already, but we should
//      have a test case for this.

use crate::xml::XmlPropertyType;

/// A "trivial" conversion between an XML attribute and a `String`. When the attribute value
/// is missing, returns a "Value missing." error.
///
/// ## Specification
///  - Section 3.1.1
impl XmlPropertyType for String {
    fn try_read(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => Ok(Some(value.to_string())),
            None => Ok(None),
        }
    }

    fn write(&self) -> Option<String> {
        Some(self.clone())
    }
}

// /// A "trivial" conversion between an XML attribute and an optional `String`. The `None` value
// /// maps to a missing XML attribute.
// ///
// /// ## Specification
// ///  - Section 3.1.1
// impl XmlPropertyType for Option<String> {
//     fn try_read(value: Option<&str>) -> Result<Option<Self>, String> {
//         Ok(value.map(|it| it.to_string()))
//     }

//     fn write(&self) -> Option<String> {
//         self.clone()
//     }
// }

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
    fn try_read(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some("1") | Some("true") => Ok(Some(true)),
            Some("0") | Some("false") => Ok(Some(false)),
            Some(value) => Err(format!(
                "Value `{value}` does not represent a valid `bool`."
            )),
            None => Ok(None),
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
    fn try_read(value: Option<&str>) -> Result<Option<Self>, String> {
        if let Some(value) = value {
            match value.parse::<i32>() {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid signed integer ({}).",
                    e
                )),
            }
        } else {
            Ok(None)
        }
    }

    fn write(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}
