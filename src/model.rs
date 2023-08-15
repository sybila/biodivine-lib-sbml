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

    pub fn species(&self) -> Child<XmlList<Specie>> {
        Child::new(self.as_xml(), "listOfSpecies")
    }

    pub fn parameters(&self) -> Child<XmlList<Parameter>> {
        Child::new(self.as_xml(), "listOfParameters")
    }

    pub fn initial_assignments(&self) -> Child<XmlList<InitialAssignment>> {
        Child::new(self.as_xml(), "listOfInitialAssignments")
    }

    pub fn rules<T: Rule>(&self) -> Child<XmlList<T>> {
        Child::new(self.as_xml(), "listOfRules")
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
#[derive(Debug, Display, EnumString)]
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
    // override default behaviour (inherited from SBase) as compartment id is required.
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

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Specie(XmlElement);

impl Specie {
    // override default behaviour (inherited from SBase) as specie id is required.
    pub fn id(&self) -> Property<String> {
        Property::new(self.as_xml(), "id")
    }

    // would it be useful to return Property<Compartment> ? If so, then how to find compartment
    // by id ?
    pub fn compartment(&self) -> Property<String> {
        Property::new(self.as_xml(), "compartment")
    }

    pub fn initial_amount(&self) -> Property<Option<f64>> {
        Property::new(self.as_xml(), "initialAmount")
    }

    pub fn initial_concentration(&self) -> Property<Option<f64>> {
        Property::new(self.as_xml(), "initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    pub fn substance_units(&self) -> Property<Option<BaseUnit>> {
        Property::new(self.as_xml(), "substanceUnits")
    }

    pub fn has_only_substance_units(&self) -> Property<bool> {
        Property::new(self.as_xml(), "hasOnlySubstanceUnits")
    }

    pub fn boundary_condition(&self) -> Property<bool> {
        Property::new(self.as_xml(), "boundaryCondition")
    }

    pub fn constant(&self) -> Property<bool> {
        Property::new(self.as_xml(), "constant")
    }

    pub fn conversion_factor(&self) -> Property<Option<String>> {
        Property::new(self.as_xml(), "conversionFactor")
    }
}

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Parameter(XmlElement);

impl Parameter {
    // override default behaviour (inherited from SBase) as specie id is required.
    pub fn id(&self) -> Property<String> {
        Property::new(self.as_xml(), "id")
    }

    pub fn value(&self) -> Property<Option<f64>> {
        Property::new(self.as_xml(), "value")
    }

    pub fn units(&self) -> Property<Option<BaseUnit>> {
        Property::new(self.as_xml(), "units")
    }

    pub fn constant(&self) -> Property<bool> {
        Property::new(self.as_xml(), "constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn symbol(&self) -> Property<String> {
        Property::new(self.as_xml(), "symbol")
    }

    pub fn math(&self) -> Child<Math> {
        Child::new(self.as_xml(), "math")
    }
}

pub trait Rule : XmlWrapper {
    fn math(&self) -> Child<Math> {
        Child::new(self.as_xml(), "math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AlgebraicRule(XmlElement);

impl Rule for AlgebraicRule {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl AssignmentRule {
    pub fn variable(&self) -> Property<String> {
        Property::new(self.as_xml(), "variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl RateRule {
    pub fn variable(&self) -> Property<String> {
        Property::new(self.as_xml(), "variable")
    }
}
