use crate::xml::impl_xml_child::Child;
use crate::xml::impl_xml_property::Property;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::{SBase, XmlWrapper};
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

    pub fn function_definitions(&self) -> Child<XmlList<SbmlFunctionDefinition>> {
        Child::new(self.as_xml(), "listOfFunctionDefinitions")
    }

    pub fn unit_definitions(&self) -> Child<XmlList<SbmlUnitDefinition>> {
        Child::new(self.as_xml(), "listOfUnitDefinitions")
    }

    pub fn compartments(&self) -> Child<XmlList<Compartment>> {
        Child::new(self.as_xml(), "listOfCompartments")
    }
}

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlFunctionDefinition(XmlElement);

impl SbmlFunctionDefinition {
    pub fn math(&self) -> Child<Math> {
        Child::new(self.as_xml(), "math")
    }
}

/// A [Math] element represents an [XmlElement] related to MathML which is 
/// separated from SBML specification.
#[derive(Clone, Debug, XmlWrapper)]
pub struct Math(XmlElement);

/// Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    pub fn units(&self) -> Child<XmlList<Unit>> {
        Child::new(self.as_xml(), "listOfUnits")
    }
}
/// Unit representation
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Unit(XmlElement);

impl Unit {
    pub fn kind(&self) -> Property<BaseUnit> {
        Property::new(self.as_xml(), "kind")
    }

    pub fn exponent(&self) -> Property<f64> {
        Property::new(self.as_xml(), "exponent")
    }

    pub fn scale(&self) -> Property<i32> {
        Property::new(self.as_xml(), "scale")
    }

    pub fn multiplier(&self) -> Property<f64> {
        Property::new(self.as_xml(), "multiplier")
    }

    // pub fn set_kind(&self, value: BaseUnit) {
    //     let mut doc = self.write_doc();
    //     self.element()
    //         .set_attribute(doc.deref_mut(), "kind", value.to_string())
    // }

    // /// In following 3 functions:
    // ///     - Pass an Integer (and convert) or a String as the value for numeric attributes ?
    // ///     - If we choose passing a String, then perform input-check or assume valid input
    // ///       and leave any invalid values to be detected by some validator ?
    // pub fn set_exponent(&self, value: &String) {
    //     let mut doc = self.write_doc();
    //     self.element()
    //         .set_attribute(doc.deref_mut(), "exponent", value)
    // }

    // pub fn set_scale(&self, value: &String) {
    //     let mut doc = self.write_doc();
    //     self.element()
    //         .set_attribute(doc.deref_mut(), "scale", value)
    // }

    // pub fn set_multiplier(&self, value: &String) {
    //     let mut doc = self.write_doc();
    //     self.element()
    //         .set_attribute(doc.deref_mut(), "multiplier", value)
    // }
}

/// Set of pre-defined base units that are allowed for unit definition
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

/// Individual compartment definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Compartment(XmlElement);

impl Compartment {
    /// override default implementation as compartment id is required
    pub fn id(&self) -> Property<String> {
        Property::new(self.as_xml(), "id")
    }

    pub fn spatial_dimensions(&self) -> Property<Option<f64>> {
        Property::new(self.as_xml(), "spatialDimensions")
    }

    pub fn size(&self) -> Property<Option<f64>> {
        Property::new(self.as_xml(), "size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "units")
    }

    pub fn constant(&self) -> Property<bool> {
        Property::new(self.as_xml(), "constant")
    }
}
