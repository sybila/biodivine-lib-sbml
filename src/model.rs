use crate::constants::namespaces::NS_MATHML;
use crate::sbase::{SBase, SbmlUtils};
use crate::validation::SbmlIssue;
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredProperty, RequiredXmlProperty, XmlDefault,
    XmlDocument, XmlElement, XmlList, XmlNamedSubtype, XmlSupertype, XmlWrapper,
};
use macros::{SBase, XmlWrapper};
use std::ops::Deref;
use strum_macros::{Display, EnumString};

/// A type-safe representation of an SBML <model> element.
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Model(XmlElement);

impl XmlDefault for Model {
    fn default(document: XmlDocument) -> Self {
        Model::new_empty(document, "model")
    }
}

/// Public functions to manipulate with the contents of SBML [Model]
/// i.e., optional lists inside SBML model
impl Model {
    pub fn function_definitions(&self) -> OptionalChild<XmlList<FunctionDefinition>> {
        self.optional_sbml_child("listOfFunctionDefinitions")
    }

    pub fn unit_definitions(&self) -> OptionalChild<XmlList<UnitDefinition>> {
        self.optional_sbml_child("listOfUnitDefinitions")
    }

    pub fn compartments(&self) -> OptionalChild<XmlList<Compartment>> {
        self.optional_sbml_child("listOfCompartments")
    }

    pub fn species(&self) -> OptionalChild<XmlList<Species>> {
        self.optional_sbml_child("listOfSpecies")
    }

    pub fn parameters(&self) -> OptionalChild<XmlList<Parameter>> {
        self.optional_sbml_child("listOfParameters")
    }

    pub fn initial_assignments(&self) -> OptionalChild<XmlList<InitialAssignment>> {
        self.optional_sbml_child("listOfInitialAssignments")
    }

    pub fn rules(&self) -> OptionalChild<XmlList<AbstractRule>> {
        self.optional_sbml_child("listOfRules")
    }

    pub fn constraints(&self) -> OptionalChild<XmlList<Constraint>> {
        self.optional_sbml_child("listOfConstraints")
    }

    pub fn reactions(&self) -> OptionalChild<XmlList<Reaction>> {
        self.optional_sbml_child("listOfReactions")
    }

    pub fn events(&self) -> OptionalChild<XmlList<Event>> {
        self.optional_sbml_child("listOfEvents")
    }

    pub fn apply_rule_10102(&self, _issues: &mut Vec<SbmlIssue>) {
        let _rule_number = "10102".to_string();
        let _doc = self.document().read().unwrap().deref();

        todo!()
    }
}

/// Individual function definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionDefinition(XmlElement);

impl FunctionDefinition {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

impl XmlDefault for FunctionDefinition {
    fn default(document: XmlDocument) -> Self {
        FunctionDefinition::new_empty(document, "functionDefinition")
    }
}

/// A [Math] element represents an [XmlElement] related to MathML which is
/// separated from SBML specification.
#[derive(Clone, Debug, XmlWrapper)]
pub struct Math(XmlElement);

impl XmlDefault for Math {
    fn default(document: XmlDocument) -> Self {
        unsafe { Math::unchecked_cast(XmlElement::new_quantified(document, "math", NS_MATHML)) }
    }
}
/// Individual unit definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct UnitDefinition(XmlElement);

impl UnitDefinition {
    pub fn units(&self) -> OptionalChild<XmlList<Unit>> {
        self.optional_sbml_child("listOfUnits")
    }
}

impl XmlDefault for UnitDefinition {
    fn default(document: XmlDocument) -> Self {
        UnitDefinition::new_empty(document, "unitDefinition")
    }
}

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

/// Individual compartment definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Compartment(XmlElement);

impl XmlDefault for Compartment {
    fn default(document: XmlDocument) -> Self {
        Compartment::new(document, true)
    }
}

impl Compartment {
    pub fn new(document: XmlDocument, is_constant: bool) -> Self {
        let cmp = Compartment::new_empty(document, "compartment");
        cmp.constant().set(&is_constant);
        cmp
    }

    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn spatial_dimensions(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("spatialDimensions")
    }

    pub fn size(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("size")
    }

    /// TODO: implement units lookup in model according to documentation
    pub fn units(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}

/// Individual specie definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Species(XmlElement);

impl Species {

    pub fn new(document: XmlDocument, id: &String, compartment: &String) -> Self {
        let obj = Species::new_empty(document, "species");
        obj.id().set(id);
        obj.compartment().set(compartment);
        obj.has_only_substance_units().set(&false);
        obj.boundary_condition().set(&true);
        obj.constant().set(&true);
        obj
    }

    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn compartment(&self) -> RequiredProperty<String> {
        self.required_sbml_property("compartment")
    }

    pub fn initial_amount(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialAmount")
    }

    pub fn initial_concentration(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("initialConcentration")
    }

    // TODO: need to embrace recommended units (p. 148)
    pub fn substance_units(&self) -> OptionalProperty<BaseUnit> {
        self.optional_sbml_property("substanceUnits")
    }

    pub fn has_only_substance_units(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("hasOnlySubstanceUnits")
    }

    pub fn boundary_condition(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("boundaryCondition")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }

    pub fn conversion_factor(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("conversionFactor")
    }
}

/// Individual parameter definition
#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Parameter(XmlElement);

impl Parameter {
    pub fn new(document: XmlDocument, id: &String, constant: bool) -> Self {
        let obj = Parameter::new_empty(document, "parameter");
        obj.id().set(id);
        obj.constant().set(&constant);
        obj
    }

    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalProperty<BaseUnit> {
        self.optional_sbml_property("units")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct InitialAssignment(XmlElement);

impl InitialAssignment {
    pub fn new(document: XmlDocument, symbol: &String) -> Self {
        let obj = InitialAssignment::new_empty(document, "initialAssignment");
        obj.symbol().set(symbol);
        obj
    }

    pub fn symbol(&self) -> RequiredProperty<String> {
        self.required_sbml_property("symbol")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

pub enum RuleTypes {
    // Other is used to represent rules that are only defined in (hypothetical) SBML extensions
    // that are not covered by this library.
    Other(AbstractRule),
    Algebraic(AlgebraicRule),
    Assignment(AssignmentRule),
    Rate(RateRule),
}

pub trait Rule: SBase {
    fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AbstractRule(XmlElement);

impl Rule for AbstractRule {}

impl XmlSupertype for AbstractRule {}

impl AbstractRule {
    pub fn cast(self) -> RuleTypes {
        if let Some(rule) = self.try_downcast::<AlgebraicRule>() {
            RuleTypes::Algebraic(rule)
        } else if let Some(rule) = self.try_downcast::<AssignmentRule>() {
            RuleTypes::Assignment(rule)
        } else if let Some(rule) = self.try_downcast::<RateRule>() {
            RuleTypes::Rate(rule)
        } else {
            RuleTypes::Other(self)
        }
    }

    pub fn default(document: XmlDocument, tag_name: &str) -> Self {
        AbstractRule::new_empty(document, tag_name)
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AlgebraicRule(XmlElement);

impl XmlDefault for AlgebraicRule {
    fn default(document: XmlDocument) -> Self {
        AlgebraicRule::new_empty(document, "algebraicRule")
    }
}
impl Rule for AlgebraicRule {}

impl XmlNamedSubtype<AbstractRule> for AlgebraicRule {
    fn expected_tag_name() -> &'static str {
        "algebraicRule"
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct AssignmentRule(XmlElement);

impl Rule for AssignmentRule {}

impl XmlNamedSubtype<AbstractRule> for AssignmentRule {
    fn expected_tag_name() -> &'static str {
        "assignmentRule"
    }
}

impl AssignmentRule {
    pub fn new(document: XmlDocument, variable: &String) -> Self {
        let obj = AssignmentRule::new_empty(document, "assignmentRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredProperty<String> {
        self.required_sbml_property("variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct RateRule(XmlElement);

impl Rule for RateRule {}

impl XmlNamedSubtype<AbstractRule> for RateRule {
    fn expected_tag_name() -> &'static str {
        "rateRule"
    }
}

impl RateRule {
    pub fn new(document: XmlDocument, variable: &String) -> Self {
        let obj = RateRule::new_empty(document, "rateRule");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredProperty<String> {
        self.required_sbml_property("variable")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Constraint(XmlElement);

impl XmlDefault for Constraint {
    fn default(document: XmlDocument) -> Self {
        Constraint::new_empty(document, "constraint")
    }
}

impl Constraint {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    pub fn message(&self) -> OptionalChild<XmlElement> {
        self.optional_html_child("message")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Reaction(XmlElement);

impl Reaction {
    pub fn new(document: XmlDocument, id: &String, reversible: bool) -> Self {
        let obj = Reaction::new_empty(document, "reaction");
        obj.id().set(id);
        obj.reversible().set(&reversible);
        obj
    }
    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn reversible(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("reversible")
    }

    pub fn compartment(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("compartment")
    }

    pub fn reactants(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfReactants")
    }

    pub fn products(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfProducts")
    }

    pub fn modifiers(&self) -> OptionalChild<XmlList<ModifierSpeciesReference>> {
        self.optional_sbml_child("listOfModifiers")
    }

    pub fn kinetic_law(&self) -> OptionalChild<KineticLaw> {
        self.optional_sbml_child("kineticLaw")
    }
}

pub trait SimpleSpeciesReference: SBase {
    fn species(&self) -> RequiredProperty<String> {
        self.required_sbml_property("species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesReference(XmlElement);

impl XmlDefault for SpeciesReference {
    fn default(document: XmlDocument) -> Self {
        SpeciesReference::new(document, true)
    }
}
impl SimpleSpeciesReference for SpeciesReference {}

impl SpeciesReference {
    pub fn new(document: XmlDocument, constant: bool) -> Self {
        let obj = SpeciesReference::new_empty(document, "speciesReference");
        obj.constant().set(&constant);
        obj
    }

    pub fn stoichiometry(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("stoichiometry")
    }

    pub fn constant(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ModifierSpeciesReference(XmlElement);

impl XmlDefault for ModifierSpeciesReference {
    fn default(document: XmlDocument) -> Self {
        ModifierSpeciesReference::new_empty(document, "modifierSpeciesReference")
    }
}

impl SimpleSpeciesReference for ModifierSpeciesReference {}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct KineticLaw(XmlElement);

impl XmlDefault for KineticLaw {
    fn default(document: XmlDocument) -> Self {
        KineticLaw::new_empty(document, "kineticLaw")
    }
}

impl KineticLaw {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    pub fn local_parameters(&self) -> OptionalChild<XmlList<LocalParameter>> {
        self.optional_sbml_child("listOfLocalParameters")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct LocalParameter(XmlElement);

impl LocalParameter {
    pub fn new(document: XmlDocument, id: &String) -> Self {
        let obj = LocalParameter::new_empty(document, "localParameter");
        obj.id().set(id);
        obj
    }

    pub fn id(&self) -> RequiredProperty<String> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalProperty<String> {
        self.optional_sbml_property("units")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Event(XmlElement);

impl XmlDefault for Event {
    fn default(document: XmlDocument) -> Self {
        Event::new(document, false)
    }
}
impl Event {
    pub fn new(document: XmlDocument, use_values_from_trigger_time: bool) -> Self {
        let obj = Event::new_empty(document, "event");
        obj.use_values_from_trigger_time().set(&use_values_from_trigger_time);
        obj
    }

    pub fn use_values_from_trigger_time(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("useValuesFromTriggerTime")
    }

    pub fn trigger(&self) -> OptionalChild<Trigger> {
        self.optional_sbml_child("trigger")
    }

    pub fn priority(&self) -> OptionalChild<Priority> {
        self.optional_sbml_child("priority")
    }

    pub fn delay(&self) -> OptionalChild<Delay> {
        self.optional_sbml_child("delay")
    }

    pub fn event_assignments(&self) -> OptionalChild<XmlList<EventAssignment>> {
        self.optional_sbml_child("listOfEventAssignments")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Trigger(XmlElement);

impl XmlDefault for Trigger {
    fn default(document: XmlDocument) -> Self {
        Trigger::new(document, false, false)
    }
}

impl Trigger {
    pub fn new(document: XmlDocument, persistent: bool, initial_value: bool) -> Self {
        let obj = Trigger::new_empty(document, "trigger");
        obj.persistent().set(&persistent);
        obj.initial_value().set(&initial_value);
        obj
    }

    pub fn initial_value(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("initialValue")
    }

    pub fn persistent(&self) -> RequiredProperty<bool> {
        self.required_sbml_property("persistent")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Priority(XmlElement);

impl XmlDefault for Priority {
    fn default(document: XmlDocument) -> Self {
        Priority::new_empty(document, "priority")
    }
}

impl Priority {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Delay(XmlElement);

impl XmlDefault for Delay {
    fn default(document: XmlDocument) -> Self {
        Delay::new_empty(document, "delay")
    }
}

impl Delay {
    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct EventAssignment(XmlElement);

impl EventAssignment {
    pub fn new(document: XmlDocument, variable: &String) -> Self {
        let obj = EventAssignment::new_empty(document, "eventAssignment");
        obj.variable().set(variable);
        obj
    }

    pub fn variable(&self) -> RequiredProperty<String> {
        self.required_sbml_property("variable")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }
}
