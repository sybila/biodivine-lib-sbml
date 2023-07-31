use xml_doc::Element;
use crate::sbase::SBaseDefault;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug)]
pub struct SbmlModel {
    xml: XmlElement,
}

impl XmlWrapper for SbmlModel {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlModel {
    fn from(xml: XmlElement) -> Self {
        SbmlModel { xml }
    }
}

/// Adds the default implementation of [SBase] to the [SbmlModel].
/// Allows to get/set id, name, etc. of [SbmlModel]
impl SBaseDefault for SbmlModel {}

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new()
impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel { xml }
    }

    pub fn get_function_definitions(&self) -> XmlList<SbmlFunctionDefinition> {
        let list_element = {
            let xml = self.read_doc();
            self.element()
                .find(xml.deref(), "listOfFunctionDefinitions")
                .unwrap()
        };
        XmlList::from(self.as_xml().derive(list_element))
    }

    pub fn get_unit_definitions(&self) -> XmlList<SbmlUnitDefinition> {
        // possible better-readability & no copy-paste approach
        let list = self.child_element("listOfUnitDefinitions");
        XmlList::from(self.as_xml().derive(list))
    }
}

/// 1.) Function definition data type
/// Rename to just "FunctionDefinition" in accordance with the documentation ?
#[derive(Clone, Debug)]
pub struct SbmlFunctionDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlFunctionDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlFunctionDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlFunctionDefinition { xml }
    }
}

impl SBaseDefault for SbmlFunctionDefinition {}

impl SbmlFunctionDefinition {
    /// Return optional [Element] or better optional [XmlELement] ?
    /// What to do next with Math element ? Create a new struct for Math
    /// elements and provide API for get/set functionality ?
    pub fn get_math(&self) -> Option<Element> {
        let doc = self.read_doc();
        self.element().find(doc.deref(), "math")
    }

    pub fn set_math(&self, value: Element) {
        let mut doc = self.write_doc();
        match &self.element().find(doc.deref(), "math") {
            Some(mut _math) => _math = value, // valid ?
            None => self
                .element()
                .push_child(doc.deref_mut(), value.as_node())
                .unwrap(),
        }
    }
}
/// 2.) Unit definition data type
/// Rename to just "UnitDefinition" in accordance with the documentation ?
#[derive(Clone, Debug)]
pub struct SbmlUnitDefinition {
    xml: XmlElement,
}

impl XmlWrapper for SbmlUnitDefinition {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for SbmlUnitDefinition {
    fn from(xml: XmlElement) -> Self {
        SbmlUnitDefinition { xml }
    }
}

impl SBaseDefault for SbmlUnitDefinition {}

impl SbmlUnitDefinition {
    /// Get inner list of [Unit] elements.
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

/// 2.1.) Unit data type
pub struct Unit {
    xml: XmlElement,
}

impl XmlWrapper for Unit {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for Unit {
    fn from(xml: XmlElement) -> Self {
        Unit { xml }
    }
}

impl SBaseDefault for Unit {}

impl Unit {
    pub fn get_kind(&self) -> BaseUnit {
        let doc = self.read_doc();
        let raw_kind = self
            .element()
            .attribute(doc.deref(), "kind")
            .unwrap()
            .to_string();
        BaseUnit::from_str(&raw_kind).unwrap()
    }

    /// In following 3 functions:
    ///     - Return String or integer when numeric values ?
    ///     - Probably required attributes
    pub fn get_exponent(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "exponent")
            .unwrap()
            .to_string()
    }

    pub fn get_scale(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "scale")
            .unwrap()
            .to_string()
    }

    pub fn get_multiplier(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "multiplier")
            .unwrap()
            .to_string()
    }

    pub fn set_kind(&self, value: BaseUnit) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), "kind", value.to_string())
    }

    /// In following 3 functions:
    ///     - Pass an Integer (and convert) or a String as the value for numeric attributes ?
    ///     - If we choose passing a String, then perform input-check or assume valid input
    ///       and leave any invalid values to be detected by some validator ?
    pub fn set_exponent(&self, value: &String) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), "exponent", value)
    }

    pub fn set_scale(&self, value: &String) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), "scale", value)
    }

    pub fn set_multiplier(&self, value: &String) {
        let mut doc = self.write_doc();
        self.element()
            .set_attribute(doc.deref_mut(), "multiplier", value)
    }
}

#[derive(Display, EnumString)]
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

/// 3.) Compartment data type
pub struct Compartment {
    xml: XmlElement,
}

impl XmlWrapper for Compartment {
    fn as_xml(&self) -> &XmlElement {
        &self.xml
    }
}

impl From<XmlElement> for Compartment {
    fn from(xml: XmlElement) -> Self {
        Compartment { xml }
    }
}

impl SBaseDefault for Compartment {}

impl Compartment {
    /// override default implementation as compartment id is required
    pub fn get_id(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "id")
            .map(|it| it.to_string())
            .unwrap()
    }

    /// return String or Double ?
    pub fn get_spatial_dimensions(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "spatialDimensions")
            .map(|it| it.to_string())
    }

    /// return String or Double
    pub fn get_size(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "size")
            .map(|it| it.to_string())
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn get_units(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "units")
            .map(|it| it.to_string())
    }

    /// return String or Boolean ?
    pub fn get_constant(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "constant")
            .map(|it| it.to_string())
    }
}
