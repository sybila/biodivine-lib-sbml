use xml_doc::Element;

use crate::sbase::SBaseDefault;
use crate::xml::{XmlElement, XmlList, XmlWrapper};
use macros::XmlWrapper;
use std::ops::{Deref, DerefMut};
use strum_macros::{EnumString, Display};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlModel(XmlElement);

/// Adds the default implementation of [SBase] to the [SbmlModel].
/// Allows to get/set id, name, etc. of [SbmlModel]
impl SBaseDefault for SbmlModel {}

/// Public functions to manipulate with the contents of [SbmlModel]
/// i.e., optional lists inside SBML model + constructor new()
impl SbmlModel {
    pub fn new(xml: XmlElement) -> SbmlModel {
        SbmlModel::from(xml)
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

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlFunctionDefinition(XmlElement);

#[derive(Clone, Debug, XmlWrapper)]
pub struct SbmlUnitDefinition(XmlElement);

impl SbmlUnitDefinition {
    /// Get inner list of [Unit] elements.
    pub fn get_units(&self) -> XmlList<Unit> {
        let list = self.child_element("listOfUnits");
        XmlList::from(self.as_xml().derive(list))
    }
}

#[derive(Clone, Debug, XmlWrapper)]
pub struct Unit(XmlElement);

impl Unit {
    /// TODO: create an enum of reserved words for a [kind] and make it a return type (documentation p. 43)
    pub fn get_kind(&self) -> String {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "kind")
            .unwrap()
            .to_string()
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

    /// TODO: pass enum type parameter of reserver words as a value
    pub fn set_kind(&self, value: &String) {
        let mut doc = self.write_doc();
        self.element().set_attribute(doc.deref_mut(), "kind", value)
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
enum BaseUnit {
    #[strum(serialize="ampere")]
    Ampere,
    #[strum(serialize="avogadro")]
    Avogadro,
    #[strum(serialize="becquerel")]
    Becquerel,
    #[strum(serialize="candela")]
    Candela,
    #[strum(serialize="coulomb")]
    Coulomb,
    #[strum(serialize="dimensionless")]
    Dimensionless,
    #[strum(serialize="farad")]
    Farad,
    #[strum(serialize="gram")]
    Gram,
    #[strum(serialize="gray")]
    Gray,
    #[strum(serialize="hertz")]
    Hertz,
    #[strum(serialize="henry")]
    Henry,
    #[strum(serialize="item")]
    Item,
    #[strum(serialize="joule")]
    Joule,
    #[strum(serialize="katal")]
    Katal,
    #[strum(serialize="kelvin")]
    Kelvin,
    #[strum(serialize="kilogram")]
    Kilogram,
    #[strum(serialize="litre")]
    Litre,
    #[strum(serialize="lumen")]
    Lumen,
    #[strum(serialize="lux")]
    Lux,
    #[strum(serialize="metre")]
    Metre,
    #[strum(serialize="mole")]
    Mole,
    #[strum(serialize="newton")]
    Newton,
    #[strum(serialize="ohm")]
    Ohm,
    #[strum(serialize="pascal")]
    Pascal,
    #[strum(serialize="radian")]
    Radian,
    #[strum(serialize="second")]
    Second,
    #[strum(serialize="siemens")]
    Siemens,
    #[strum(serialize="sievert")]
    Sievert,
    #[strum(serialize="steradian")]
    Steradian,
    #[strum(serialize="tesla")]
    Tesla,
    #[strum(serialize="volt")]
    Volt,
    #[strum(serialize="watt")]
    Watt,
    #[strum(serialize="weber")]
    Weber,
}
