use crate::core::sbase::SbmlUtils;
use crate::xml::{
    RequiredProperty, RequiredXmlProperty, XmlDefault, XmlDocument, XmlElement, XmlPropertyType,
};
use macros::{SBase, XmlWrapper};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Unit representation
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Unit(XmlElement);

impl Unit {
    pub fn kind(&self) -> RequiredProperty<BaseUnit> {
        self.required_sbml_property("kind")
    }

    pub fn exponent(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("exponent")
    }

    pub fn scale(&self) -> RequiredProperty<i32> {
        self.required_sbml_property("scale")
    }

    pub fn multiplier(&self) -> RequiredProperty<f64> {
        self.required_sbml_property("multiplier")
    }
}

impl XmlDefault for Unit {
    fn default(document: XmlDocument) -> Self {
        let unit = Unit::new_empty(document, "unit");

        unit.kind().set(&BaseUnit::Dimensionless);
        unit.multiplier().set(&1.0);
        unit.scale().set(&0);
        unit.exponent().set(&1.0);
        unit
    }
}

/// Set of pre-defined base units that are allowed for unit definition
#[derive(Debug, Display, EnumString, PartialEq)]
pub enum BaseUnit {
    #[strum(serialize = "ampere")]
    Ampere,
    #[strum(serialize = "avogadro")]
    Avogadro,
    #[strum(serialize = "becquerel")]
    Becquerel,
    #[strum(serialize = "candela")]
    Candela,
    #[strum(serialize = "coulomb")]
    Coulomb,
    #[strum(serialize = "dimensionless")]
    Dimensionless,
    #[strum(serialize = "farad")]
    Farad,
    #[strum(serialize = "gram")]
    Gram,
    #[strum(serialize = "gray")]
    Gray,
    #[strum(serialize = "hertz")]
    Hertz,
    #[strum(serialize = "henry")]
    Henry,
    #[strum(serialize = "item")]
    Item,
    #[strum(serialize = "joule")]
    Joule,
    #[strum(serialize = "katal")]
    Katal,
    #[strum(serialize = "kelvin")]
    Kelvin,
    #[strum(serialize = "kilogram")]
    Kilogram,
    #[strum(serialize = "litre")]
    Litre,
    #[strum(serialize = "lumen")]
    Lumen,
    #[strum(serialize = "lux")]
    Lux,
    #[strum(serialize = "metre")]
    Metre,
    #[strum(serialize = "mole")]
    Mole,
    #[strum(serialize = "newton")]
    Newton,
    #[strum(serialize = "ohm")]
    Ohm,
    #[strum(serialize = "pascal")]
    Pascal,
    #[strum(serialize = "radian")]
    Radian,
    #[strum(serialize = "second")]
    Second,
    #[strum(serialize = "siemens")]
    Siemens,
    #[strum(serialize = "sievert")]
    Sievert,
    #[strum(serialize = "steradian")]
    Steradian,
    #[strum(serialize = "tesla")]
    Tesla,
    #[strum(serialize = "volt")]
    Volt,
    #[strum(serialize = "watt")]
    Watt,
    #[strum(serialize = "weber")]
    Weber,
}

/// A conversion between an XML attribute and a [BaseUnit] value. Missing attribute value is
/// interpreted as an error.
///
/// ## Specification
///  - Section 4.4.2
impl XmlPropertyType for BaseUnit {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match BaseUnit::from_str(value) {
                Ok(unit) => Ok(Some(unit)),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid base unit ({})",
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
