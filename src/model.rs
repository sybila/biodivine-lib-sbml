use crate::constants::namespaces::{NS_SBML_CORE, URL_HTML, URL_MATHML, URL_SBML_CORE};
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredProperty, XmlDefault, XmlDocument, XmlElement,
    XmlList, XmlWrapper,
};
use macros::{SBase, XmlWrapper};
use strum_macros::{Display, EnumString};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Model(XmlElement);

impl XmlDefault for Model {
    fn default(document: XmlDocument) -> Self {
        unsafe {
            Model::unchecked_cast(XmlElement::new_quantified(document, "model", NS_SBML_CORE))
        }
    }
}

/// Public functions to manipulate with the contents of SBML [Model]
/// i.e., optional lists inside SBML model
impl Model {
    pub fn function_definitions(&self) -> OptionalChild<XmlList<FunctionDefinition>> {
        OptionalChild::new(
            self.xml_element(),
            "listOfFunctionDefinitions",
            URL_SBML_CORE,
        )
    }

    pub fn unit_definitions(&self) -> OptionalChild<XmlList<UnitDefinition>> {
        OptionalChild::new(self.xml_element(), "listOfUnitDefinitions", URL_SBML_CORE)
    }

    pub fn compartments(&self) -> OptionalChild<XmlList<Compartment>> {
        OptionalChild::new(self.xml_element(), "listOfCompartments", URL_SBML_CORE)
    }

    pub fn species(&self) -> OptionalChild<XmlList<Species>> {
        OptionalChild::new(self.xml_element(), "listOfSpecies", URL_SBML_CORE)
    }

    pub fn parameters(&self) -> OptionalChild<XmlList<Parameter>> {
        OptionalChild::new(self.xml_element(), "listOfParameters", URL_SBML_CORE)
    }

    pub fn initial_assignments(&self) -> OptionalChild<XmlList<InitialAssignment>> {
        OptionalChild::new(
            self.xml_element(),
            "listOfInitialAssignments",
            URL_SBML_CORE,
        )
    }

    pub fn rules<T: Rule>(&self) -> OptionalChild<XmlList<T>> {
        OptionalChild::new(self.xml_element(), "listOfRules", URL_SBML_CORE)
    }

    pub fn constraints(&self) -> OptionalChild<XmlList<Constraint>> {
        OptionalChild::new(self.xml_element(), "listOfConstraints", URL_SBML_CORE)
    }

    pub fn reactions(&self) -> OptionalChild<XmlList<Reaction>> {
        OptionalChild::new(self.xml_element(), "listOfReactions", URL_SBML_CORE)
    }

    pub fn events(&self) -> OptionalChild<XmlList<Event>> {
        OptionalChild::new(self.xml_element(), "listOfEvents", URL_SBML_CORE)
    }
}

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

/// A [Math] element represents an [XmlElement] related to MathML which is
/// separated from SBML specification.
#[derive(Clone, Debug, XmlWrapper)]
pub struct Math(XmlElement);

/// Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct UnitDefinition(XmlElement);

impl UnitDefinition {
    pub fn units(&self) -> OptionalChild<XmlList<Unit>> {
        OptionalChild::new(self.xml_element(), "listOfUnits", URL_SBML_CORE)
    }
}

/// Unit representation
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Unit(XmlElement);

impl Unit {
    pub fn kind(&self) -> RequiredProperty<BaseUnit> {
        RequiredProperty::new(self.xml_element(), "kind")
    }

    pub fn exponent(&self) -> RequiredProperty<f64> {
        RequiredProperty::new(self.xml_element(), "exponent")
    }

    pub fn scale(&self) -> RequiredProperty<i32> {
        RequiredProperty::new(self.xml_element(), "scale")
    }

    pub fn multiplier(&self) -> RequiredProperty<f64> {
        RequiredProperty::new(self.xml_element(), "multiplier")
    }
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

impl XmlDefault for Compartment {
    fn default(document: XmlDocument) -> Self {
        unsafe {
            Compartment::unchecked_cast(XmlElement::new_quantified(
                document,
                "compartment",
                NS_SBML_CORE,
            ))
        }
    }
}

impl Compartment {
    pub fn id(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "id")
    }

    pub fn spatial_dimensions(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "spatialDimensions")
    }

    pub fn size(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xml_element(), "units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "constant")
    }
}

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Species(XmlElement);

impl Species {
    pub fn id(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "id")
    }

    pub fn compartment(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "compartment")
    }

    pub fn initial_amount(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "initialAmount")
    }

    pub fn initial_concentration(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    pub fn substance_units(&self) -> OptionalProperty<BaseUnit> {
        OptionalProperty::new(self.xml_element(), "substanceUnits")
    }

    pub fn has_only_substance_units(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "hasOnlySubstanceUnits")
    }

    pub fn boundary_condition(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "boundaryCondition")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "constant")
    }

    pub fn conversion_factor(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xml_element(), "conversionFactor")
    }
}

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Parameter(XmlElement);

impl Parameter {
    pub fn id(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "value")
    }

    pub fn units(&self) -> OptionalProperty<BaseUnit> {
        OptionalProperty::new(self.xml_element(), "units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn symbol(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

pub trait Rule: XmlWrapper {
    fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AlgebraicRule(XmlElement);

impl Rule for AlgebraicRule {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl AssignmentRule {
    pub fn variable(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl RateRule {
    pub fn variable(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Constraint(XmlElement);

impl Constraint {
    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }

    pub fn message(&self) -> OptionalChild<XmlElement> {
        OptionalChild::new(self.xml_element(), "message", URL_HTML)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Reaction(XmlElement);

impl Reaction {
    pub fn id(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "id")
    }

    pub fn reversible(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "reversible")
    }

    pub fn compartment(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xml_element(), "compartment")
    }

    pub fn reactants(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        OptionalChild::new(self.xml_element(), "listOfReactants", URL_SBML_CORE)
    }

    pub fn products(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        OptionalChild::new(self.xml_element(), "listOfProducts", URL_SBML_CORE)
    }

    pub fn modifiers(&self) -> OptionalChild<XmlList<ModifierSpeciesReference>> {
        OptionalChild::new(self.xml_element(), "listOfModifiers", URL_SBML_CORE)
    }

    pub fn kinetic_law(&self) -> OptionalChild<KineticLaw> {
        OptionalChild::new(self.xml_element(), "kineticLaw", URL_SBML_CORE)
    }
}

trait SimpleSpeciesReference: XmlWrapper {
    fn species(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesReference(XmlElement);

impl SimpleSpeciesReference for SpeciesReference {}

impl SpeciesReference {
    pub fn stoichiometry(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "stoichiometry")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ModifierSpeciesReference(XmlElement);

impl SimpleSpeciesReference for ModifierSpeciesReference {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct KineticLaw(XmlElement);

impl KineticLaw {
    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }

    pub fn local_parameters(&self) -> OptionalChild<XmlList<LocalParameter>> {
        OptionalChild::new(self.xml_element(), "listOfLocalParameters", URL_SBML_CORE)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct LocalParameter(XmlElement);

impl LocalParameter {
    pub fn id(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        OptionalProperty::new(self.xml_element(), "value")
    }

    pub fn units(&self) -> OptionalProperty<String> {
        OptionalProperty::new(self.xml_element(), "units")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Event(XmlElement);

impl Event {
    pub fn use_values_from_trigger_time(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "useValuesFromTriggerTime")
    }

    pub fn trigger(&self) -> OptionalChild<Trigger> {
        OptionalChild::new(self.xml_element(), "trigger", URL_SBML_CORE)
    }

    pub fn priority(&self) -> OptionalChild<Priority> {
        OptionalChild::new(self.xml_element(), "priority", URL_SBML_CORE)
    }

    pub fn delay(&self) -> OptionalChild<Delay> {
        OptionalChild::new(self.xml_element(), "delay", URL_SBML_CORE)
    }

    pub fn event_assignments(&self) -> OptionalChild<XmlList<EventAssignment>> {
        OptionalChild::new(self.xml_element(), "listOfEventAssignments", URL_SBML_CORE)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Trigger(XmlElement);

impl Trigger {
    pub fn initial_value(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "initialValue")
    }

    pub fn persistent(&self) -> RequiredProperty<bool> {
        RequiredProperty::new(self.xml_element(), "persistent")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Priority(XmlElement);

impl Priority {
    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Delay(XmlElement);

impl Delay {
    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct EventAssignment(XmlElement);

impl EventAssignment {
    pub fn variable(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "value")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        OptionalChild::new(self.xml_element(), "math", URL_MATHML)
    }
}
