use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlElement, XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// Unit representation
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "unit", namespace = NS_SBML_CORE)]
pub struct Unit(XmlElement);

impl Unit {
    /// The `kind` attribute of this `<unit>` element.
    pub fn kind(&self) -> RequiredSbmlProperty<'_, BaseUnit> {
        self.required_sbml_property("kind")
    }

    /// The `exponent` attribute of this `<unit>` element.
    pub fn exponent(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_sbml_property("exponent")
    }

    /// The `scale` attribute of this `<unit>` element.
    pub fn scale(&self) -> RequiredSbmlProperty<'_, u32> {
        self.required_sbml_property("scale")
    }

    /// The `multiplier` attribute of this `<unit>` element.
    pub fn multiplier(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_sbml_property("multiplier")
    }
}

impl XmlDefault for Unit {
    fn default(document: Document) -> Self {
        let unit = Unit::new_empty(document);

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
    /// The `ampere` value.
    #[strum(serialize = "ampere")]
    Ampere,
    /// The `avogadro` value.
    #[strum(serialize = "avogadro")]
    Avogadro,
    /// The `becquerel` value.
    #[strum(serialize = "becquerel")]
    Becquerel,
    /// The `candela` value.
    #[strum(serialize = "candela")]
    Candela,
    /// The `coulomb` value.
    #[strum(serialize = "coulomb")]
    Coulomb,
    /// The `dimensionless` value.
    #[strum(serialize = "dimensionless")]
    Dimensionless,
    /// The `farad` value.
    #[strum(serialize = "farad")]
    Farad,
    /// The `gram` value.
    #[strum(serialize = "gram")]
    Gram,
    /// The `gray` value.
    #[strum(serialize = "gray")]
    Gray,
    /// The `hertz` value.
    #[strum(serialize = "hertz")]
    Hertz,
    /// The `henry` value.
    #[strum(serialize = "henry")]
    Henry,
    /// The `item` value.
    #[strum(serialize = "item")]
    Item,
    /// The `joule` value.
    #[strum(serialize = "joule")]
    Joule,
    /// The `katal` value.
    #[strum(serialize = "katal")]
    Katal,
    /// The `kelvin` value.
    #[strum(serialize = "kelvin")]
    Kelvin,
    /// The `kilogram` value.
    #[strum(serialize = "kilogram")]
    Kilogram,
    /// The `litre` value.
    #[strum(serialize = "litre")]
    Litre,
    /// The `lumen` value.
    #[strum(serialize = "lumen")]
    Lumen,
    /// The `lux` value.
    #[strum(serialize = "lux")]
    Lux,
    /// The `metre` value.
    #[strum(serialize = "metre")]
    Metre,
    /// The `mole` value.
    #[strum(serialize = "mole")]
    Mole,
    /// The `newton` value.
    #[strum(serialize = "newton")]
    Newton,
    /// The `ohm` value.
    #[strum(serialize = "ohm")]
    Ohm,
    /// The `pascal` value.
    #[strum(serialize = "pascal")]
    Pascal,
    /// The `radian` value.
    #[strum(serialize = "radian")]
    Radian,
    /// The `second` value.
    #[strum(serialize = "second")]
    Second,
    /// The `siemens` value.
    #[strum(serialize = "siemens")]
    Siemens,
    /// The `sievert` value.
    #[strum(serialize = "sievert")]
    Sievert,
    /// The `steradian` value.
    #[strum(serialize = "steradian")]
    Steradian,
    /// The `tesla` value.
    #[strum(serialize = "tesla")]
    Tesla,
    /// The `volt` value.
    #[strum(serialize = "volt")]
    Volt,
    /// The `watt` value.
    #[strum(serialize = "watt")]
    Watt,
    /// The `weber` value.
    #[strum(serialize = "weber")]
    Weber,
}

impl From<BaseUnit> for SId {
    fn from(value: BaseUnit) -> Self {
        // This should never fail because we know the strings used as units.
        SId::try_from(value.to_string()).unwrap()
    }
}

/// A conversion between an XML attribute and a [BaseUnit] value.
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
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}
