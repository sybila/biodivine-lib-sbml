use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlChild, XmlWrapper};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlModel(XmlElement);

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new()
impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
    }

    pub fn function_definitions(&self) -> ListOfFunctionDefinitions {
        ListOfFunctionDefinitions::for_element(self.as_xml())
    }

    pub fn unit_definitions(&self) -> ListOfUnitDefinitions {
        ListOfUnitDefinitions::for_element(self.as_xml())
    }

    pub fn compartments(&self) -> ListOfCompartments {
        ListOfCompartments::for_element(self.as_xml())
    }
}

/// 1.) Optional list of SBML function definitions
#[derive(XmlChild)]
#[child(listOfFunctionDefinitions : XmlList<SbmlFunctionDefinition>)]
pub struct ListOfFunctionDefinitions<'a>(&'a XmlElement);

/// 1.1.) Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

impl SbmlFunctionDefinition {
    fn get_math(&self) -> XmlElement {
        todo!()
    }

    fn set_math(&self, value: XmlElement) {
        todo!()
    }
}

/// 2.) Optional list of SBML unit definitions
#[derive(XmlChild)]
#[child(listOfUnitDefinitions : XmlList<SbmlUnitDefinition>)]
pub struct ListOfUnitDefinitions<'a>(&'a XmlElement);

/// 2.1.) Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Unit(XmlElement);

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
    ///     - Return String or integer/double when numeric values ?
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

/// 2.2.) Set of pre-defined base units that are allowed for unit definition
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
/// 3.) Optional list of SBMl compartments
#[derive(XmlChild)]
#[child(listOfCompartments : XmlList<Compartment>)]
pub struct ListOfCompartments<'a>(&'a XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

#[derive(XmlChild)]
#[child(listOfUnits : XmlList<Unit>)]
pub struct ListOfUnits<'a>(&'a XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> ListOfUnits {
        ListOfUnits::for_element(self.as_xml())
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Compartment(XmlElement);

impl Compartment {
    /// override default implementation as compartment id is required
    pub fn id(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "id")
            .map(|it| it.to_string())
            .unwrap()
    }

    /// return String or Double ?
    pub fn spatial_dimensions(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "spatialDimensions")
            .map(|it| it.to_string())
    }

    /// return String or Double
    pub fn size(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "size")
            .map(|it| it.to_string())
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "units")
            .map(|it| it.to_string())
    }

    /// return String or Boolean ?
    pub fn constant(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "constant")
            .map(|it| it.to_string())
    }
}


