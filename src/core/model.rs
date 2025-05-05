use std::ops::Deref;

use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::{SId, SbmlUtils};
use crate::core::{
    AbstractRule, AlgebraicRule, AssignmentRule, Compartment, Constraint, Event,
    FunctionDefinition, InitialAssignment, Parameter, Reaction, Rule, SBase, Species,
    UnitDefinition,
};
use crate::layout::Layout;
use crate::xml::py::{SbmlPropertyPy, XmlChildPy};
use crate::xml::{
    OptionalChild, OptionalSbmlProperty, OptionalXmlChild, OptionalXmlProperty,
    RequiredXmlProperty, XmlDefault, XmlDocument, XmlElement, XmlList, XmlPropertyType,
    XmlSupertype, XmlWrapper,
};
use embed_doc_image::embed_doc_image;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen_derive::gen_stub_pyclass;
use sbml_macros::{PythonXmlChild, SBase, XmlWrapper};

/// The SBML model object
/// (Section 4.2; [specification](https://raw.githubusercontent.com/combine-org/combine-specifications/main/specifications/files/sbml.level-3.version-2.core.release-2.pdf)).
///
///
/// ## 4.2 Model
///
/// <!-- A minor hack to position the UML figure nicely in the page. -->
/// <style>
/// img[alt=UML] {
///     width: 80%;
///     display: block;
///     margin: 0 auto;
///     margin-top: 1rem;
///     margin-bottom: 1rem;
/// }
/// </style>
///
/// ![UML][sbml-model]
///
/// Only one instance of a [`Model`] object is allowed per instance of an SBML Level 3 Version 2
/// Core document or data stream, and it must be located inside the `<sbml> ... </sbml>` element
/// as described in Section 4.1.
///
/// Model serves as a container for components of classes
/// [`FunctionDefinition`], [`UnitDefinition`], [`Compartment`], [`Species`], [`Parameter`],
/// [`InitialAssignment`], [`Rule`], [`Constraint`], [`Reaction`] and [`Event`]. Instances of the
/// classes are placed inside instances of classes `ListOfFunctionDefinitions`,
/// `ListOfUnitDefinitions`, `ListOfCompartments`, `ListOfSpecies`, `ListOfParameters`,
/// `ListOfInitialAssignments`, `ListOfRules`, `ListOfConstraints`, `ListOfReactions`, and
/// `ListOfEvents` (here represented using [`XmlList`]). All the lists are optional, and,
/// if present, may be empty; this is semantically equivalent to omitting the list.
/// The resulting XML data object for a full model containing every possible list would have
/// the following form:
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
///     <model id="My Model">
///         <listOfFunctionDefinitions>
///             zero or more <functionDefinition> ... </functionDefinition> elements
///         </listOfFunctionDefinitions>
///         <listOfUnitDefinitions>
///             zero or more <unitDefinition> ... </unitDefinition> elements
///         </listOfUnitDefinitions>
///         <listOfCompartments>
///             zero or more <compartment> ... </compartment> elements
///         </listOfCompartments>
///         <listOfSpecies>
///             zero or more <species> ... </species> elements
///         </listOfSpecies>
///         <listOfParameters>
///             zero or more <parameter> ... </parameter> elements
///         </listOfParameters>
///         <listOfInitialAssignments>
///             zero or more <initialAssignment> ... </initialAssignment> elements
///         </listOfInitialAssignments>
///         <listOfRules>
///             zero or more elements of subclasses of Rule
///         </listOfRules>
///         <listOfConstraints>
///             zero or more <constraint> ... </constraint> elements
///         </listOfConstraints>
///         <listOfReactions>
///             zero or more <reaction> ... </reaction> elements
///         </listOfReactions>
///         <listOfEvents>
///             zero or more <event> ... </event> elements
///         </listOfEvents>
///     </model>
/// </sbml>
/// ```
///
/// Although the lists are optional, there are dependencies between SBML components such that
/// defining some components requires defining others. For example, defining a species requires
/// defining a compartment, and defining a species reference in a reaction requires defining a
/// species. Such dependencies are explained throughout specification.
///
/// ### 4.2.1 The `sboTerm` attribute
///
/// [`Model`] inherits an optional `sboTerm` attribute of type `SBOTerm` from its parent class [`SBase`] (see Section 3.1.12
/// and Section 5). When a value is given to this attribute in a [`Model`] instance, it should be
/// an SBO identifier belonging to the branch for type [`Model`] indicated in Table 6 on p. 98.
/// The relationship is of the form “the model definition is-a X”, where X is the SBO term.
/// The term chosen should be the most precise (narrow) one that captures the overall process or
/// phenomenon represented by the overall SBML model. As discussed in Section 5 on p. 91, SBO
/// labels are optional information on a model. Applications are free to ignore `sboTerm` values.
/// A model must be interpretable without the benefit of SBO labels.
///
/// ### 4.2.2 The `substanceUnits` attribute
///
/// The `substanceUnits` attribute is used to specify the unit of measurement associated with
/// substance quantities of [`Species`] objects that do not specify units explicitly. The
/// attribute’s value must be of type `UnitSIdRef` (Section 3.1.10 on p. 13). A list of recommended
/// units is given in Section 8.2.1 on p. 148. If a given [`Species`] object definition does not
/// specify its unit of substance quantity via the `substanceUnits` attribute on [`Species`]
/// (described in Section 4.6 on p. 49), then the species inherits the value of the [`Model`]
/// `substanceUnits` attribute. If the [`Model`] does not define a value for this attribute, then
/// there is no unit to inherit, and all species that do not specify individual `substanceUnits`
/// attribute values then have no declared units for their quantities. Section 4.6.4 provides more
/// information about the units of species quantities. Note that when the identifier of a species
/// appears in a model’s mathematical expressions, the unit of measurement associated with that
/// identifier is not solely determined by setting `substanceUnits` on [`Model`] or [`Species`].
/// Section 4.6.5 and Section 4.6.8 explain this point in more detail.
///
/// ### 4.2.3 The `timeUnits` attribute
///
/// The `timeUnits` attribute is used to specify the unit in which time is measured in the model.
/// The value of this attribute must be of type `UnitSIdRef` (Section 3.1.10 on p. 13). A list of
/// recommended units is given in Section 8.2.1 on p. 148.
///
/// This attribute on [`Model`] is the only way to specify a unit for time in a model. It is a
/// global attribute; time is measured in the model everywhere in the same way. This is
/// particularly relevant to [`Reaction`] and [`RateRule`] objects in a model: all [`Reaction`]
/// and [`RateRule`] objects in SBML define per-time values, and the unit of time is given by
/// the `timeUnits` attribute on the [`Model`] object instance. If the Model [`timeUnits`]
/// attribute has no value, it means that the unit of time is not defined for the model’s
/// reactions and rate rules. Leaving it unspecified in an SBML model does not result in an
/// invalid model; however, as a matter of best practice, we strongly recommend that all
/// models specify units of measurement for time.
///
/// ### 4.2.4 The `volumeUnits`, `areaUnits` and `lengthUnits` attributes
///
/// The attributes `volumeUnits`, `areaUnits` and `lengthUnits` together are used to set the units
/// of measurements for the sizes of [`Compartment`] objects in the model when those objects
/// do not otherwise specify units. The three attributes correspond to the most common cases
/// of compartment dimensions: `volumeUnits` for compartments having attribute value
/// `spatialDimensions=“3”`, `areaUnits` for compartments having `spatialDimensions=“2”`, and
/// `lengthUnits` for compartments having `spatialDimensions=“1”`. The values of these attributes
/// must be of type `UnitSIdRef` (Section 3.1.10 on p. 13). A list of recommended units is given
/// in Section 8.2.1 on p. 148. The attributes are not applicable to compartments whose
/// `spatialDimensions` attribute values are not one of `1`, `2` or `3`.
///
/// If a given [`Compartment`] object instance does not provide a value for its `units` attribute,
/// then the unit of measurement of that compartment’s size is inherited from the value specified
/// by the Model `volumeUnits`, `areaUnits` or `lengthUnits` attribute, as appropriate based on
/// the [`Compartment`] object’s `spatialDimensions` attribute value. If the [`Model`] object does
/// not define the relevant attribute, then there are no units to inherit, and all compartments
/// that do not set a value for their `units` attribute then have no units associated with their
/// compartment sizes. Section 4.5.4 provides more information about units of compartment sizes.
///
/// The use of three separate attributes is a carry-over from SBML Level 2. Note that it is
/// entirely possible for a model to define a value for two or more of the attributes `volumeUnits`,
/// `areaUnits` and `lengthUnits` simultaneously, because SBML models may contain compartments
/// with different numbers of dimensions.
///
/// ### 4.2.5 The `extentUnits` attribute
///
/// Reactions are processes that occur over time. These processes involve events of some sort,
/// where a single “reaction event” is one in which some set of entities (known as reactants,
/// products and modifiers in SBML) interact, once. The extent of a reaction is a measure of how
/// many times the reaction has occurred, while the time derivative of the extent gives the
/// instantaneous rate at which the reaction is occurring. Thus, what is colloquially referred
/// to as the “rate of the reaction” is in fact equal to the rate of change of reaction extent.
///
/// The combination of `extentUnits` and `timeUnits` defines the units of kinetic laws in SBML
/// and establishes how the numerical value of each KineticLaw’s mathematical formula
/// (Section 4.11.5 on p. 74) is meant to be interpreted in a model. The units of the kinetic laws
/// are taken to be `extentUnits` divided by `timeUnits`. A list of recommended units is given in
/// Section 8.2.1 on p. 148.
///
/// Note that this embodies an important principle in SBML models: all reactions in an SBML model
/// must have the same units for the rate of change of extent. In other words, the units of all
/// reaction rates in the model must be the same. There is only one global value for `extentUnits`
/// and one global value for `timeUnits`.
///
/// ### 4.2.6 The `conversionFactor` attribute
///
/// The attribute `conversionFactor` defines a global value inherited by all [`Species`] object
/// instances that do not define separate values for their `conversionFactor` attributes. The value
/// of this attribute must be of type `SIdRef` (Section 3.1.8 on p. 13) and refer to a
/// [`Parameter`] object instance defined in the model. The [`Parameter`] object in question must
/// be a constant; i.e., it must have its constant attribute value set to `true`.
///
/// If a given [`Species`] object definition does not specify a conversion factor via the
/// `conversionFactor` attribute on [`Species`] (described in Section 4.6 on p. 49), then the
/// species inherits the conversion factor specified by the [`Model`] `conversionFactor` attribute.
/// If the [`Model`] does not define a value for this attribute, then there is no conversion
/// factor to inherit. Section 4.11.7 on p. 77 describes how to interpret the effects of
/// reactions on species in that situation. More information about conversion factors in SBML
/// is provided in Section 4.6 on p. 49 and Section 4.11 on p. 68.
///
/// ### 4.2.7 The `ListOf` container classes
///
/// The various `ListOf___` classes are merely containers used for organizing the main components
/// of an SBML document. `ListOfFunctionDefinitions`, `ListOfUnitDefinitions`,
/// `ListOfCompartments`, `ListOfSpecies`, `ListOfParameters`, `ListOfInitialAssignments`,
/// `ListOfRules`, `ListOfConstraints`, `ListOfReactions`, and `ListOfEvents` are all derived
/// from the abstract class [`SBase`] (Section 3.2 on p. 14), and inherit `SBase`’s various
/// attributes and sub-elements. The `ListOf___` classes do not add any attributes of their own.
///
/// There are several motivations for grouping SBML components within XML elements with names of
/// the form `listOfClassNames` rather than placing all the components directly at the top level.
/// First, the fact that the container classes are derived from [`SBase`] means that software
/// tools can add information about the lists themselves into each list container’s [`Annotation`],
/// a feature that a number of today’s software tools exploit. Second, we believe the grouping
/// leads to a more modular structure that is helpful when working with elements from multiple
/// SBML Level 3 packages. Third, we believe that it makes visual reading of models in XML easier,
/// for situations when humans must inspect and edit SBML directly.
///
/// Lists are allowed to be empty for two reasons. First, this allows model writers to add
/// [`Annotation`] and [`Notes`] objects to a given list even when the list is empty in a model;
/// this can be useful, for instance, to let a modeler explain why the components are absent
/// from the model. Second, it enables SBML Level 3 package specifications to define new elements
/// to be children of these lists, and for these child elements to be used without the
/// requirement that at least one SBML Level 3 Core element always be present.
///
#[embed_doc_image("sbml-model", "docs-images/uml-model.png")]
#[derive(Clone, Debug, XmlWrapper, SBase, PythonXmlChild)]
#[pyclass]
#[gen_stub_pyclass]
pub struct Model(XmlElement);

impl XmlDefault for Model {
    fn default(document: XmlDocument) -> Self {
        Model::new_empty(document, "model")
    }
}

#[pymethods]
impl Model {
    #[pyo3(name = "id")]
    pub fn id_py(&self) -> SbmlPropertyPy {
        SbmlPropertyPy::new_optional(self.id())
    }

    #[pyo3(name = "substance_units")]
    pub fn substance_units_py(&self) -> SbmlPropertyPy {
        SbmlPropertyPy::new_optional(self.substance_units())
    }

    #[pyo3(name = "parameters")]
    pub fn parameters_py(&self) -> XmlChildPy {
        XmlChildPy::new_optional(self.parameters())
    }
}

/// The SBML-defined components of the [`Model`] class.
impl Model {
    pub fn substance_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("substanceUnits")
    }

    pub fn time_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("timeUnits")
    }

    pub fn volume_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("volumeUnits")
    }

    pub fn area_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("areaUnits")
    }

    pub fn length_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("lengthUnits")
    }

    pub fn extent_units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("extentUnits")
    }

    pub fn conversion_factor(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("conversionFactor")
    }

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

    pub fn layouts(&self) -> OptionalChild<XmlList<Layout>> {
        self.optional_package_child("listOfLayouts", NS_LAYOUT, false)
    }
}

/// Other methods for creating and manipulating SBML [`Model`].
impl Model {
    /// Try to find an instance of a [Model] element for the given child element.
    ///
    /// The child can be any SBML tag, as long as it appears in an SBML model (i.e. one of
    /// its transitive parents is a [Model] element). If this is not satisfied, the method
    /// returns `None`.
    pub fn for_child_element(child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(child, "model")
    }

    /// Returns a vector of [FunctionDefinition] identifiers (attribute **id**). Function definitions
    /// without IDs are not included in the output.
    pub(crate) fn function_definition_identifiers(&self) -> Vec<SId> {
        if let Some(function_definitions) = self.function_definitions().get() {
            function_definitions
                .iter()
                .filter_map(|def| def.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find a [FunctionDefinition] by its *id* and return a number of arguments this function expects.
    /// More precisely, find a number of **bvar** elements inside **lambda** inside **math** element of
    /// [FunctionDefinition]. If [FunctionDefinition] cannot be found or the is missing the appropriate
    /// math element, returns `None`.
    pub(crate) fn function_definition_arguments(&self, id: &str) -> Option<usize> {
        // Check that the list of a function definitions is present.
        let definitions = self.function_definitions().get()?;

        // And that we have found a function with the given id.
        let expected = Some(id.to_string());
        let function = definitions
            .iter()
            .find(|function| function.id().get().unwrap().set() == expected)?;

        // And this function has its `math` element with a `lambda` child element specified.
        let doc = self.read_doc();
        let math = function.math().get()?;
        let lambda = math.raw_element().find(doc.deref(), "lambda")?;
        let lambda = XmlElement::new_raw(self.document(), lambda);

        // We then return the number of `bvar` child nodes in the lambda element.
        let count = lambda
            .child_elements_filtered(|it| it.tag_name() == "bvar")
            .len();

        Some(count)
    }

    /// Returns a vector of [UnitDefinition] identifiers (attribute **id**). Unit definitions
    /// without IDs are not included in the output.
    pub(crate) fn unit_definition_identifiers(&self) -> Vec<SId> {
        if let Some(unit_definitions) = self.unit_definitions().get() {
            unit_definitions
                .iter()
                .filter_map(|unit| unit.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [LocalParameter] identifiers (attribute **id**).
    pub(crate) fn local_parameter_identifiers(&self) -> Vec<SId> {
        let mut identifiers: Vec<SId> = Vec::new();

        let Some(reactions) = self.reactions().get() else {
            return identifiers;
        };

        for reaction in reactions.iter() {
            let local_parameters = reaction
                .kinetic_law()
                .get()
                .and_then(|law| law.local_parameters().get());
            if let Some(local_parameters) = local_parameters {
                identifiers.extend(local_parameters.iter().map(|param| param.id().get()));
            }
        }

        identifiers
    }

    /// Returns a vector of [Species] identifiers (attribute **id**).
    pub(crate) fn species_identifiers(&self) -> Vec<SId> {
        if let Some(species) = self.species().get() {
            species.iter().map(|species| species.id().get()).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [Compartment] identifiers (attribute **id**).
    pub(crate) fn compartment_identifiers(&self) -> Vec<SId> {
        if let Some(compartment) = self.compartments().get() {
            compartment
                .iter()
                .map(|compartment| compartment.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of [Parameter] identifiers (attribute **id**).
    pub(crate) fn parameter_identifiers(&self) -> Vec<SId> {
        if let Some(parameters) = self.parameters().get() {
            parameters.iter().map(|param| param.id().get()).collect()
        } else {
            vec![]
        }
    }

    /// Returns a vector of [SpeciesReference] identifiers (attribute **id**). Unit definitions
    /// without IDs are not included in the output.
    pub(crate) fn species_reference_identifiers(&self) -> Vec<SId> {
        let mut identifiers: Vec<SId> = vec![];
        // If the list of reactions is present...
        if let Some(reactions) = self.reactions().get() {
            for reaction in reactions.as_vec() {
                // ...we extract identifiers of reactants and products.
                for list in &[reaction.reactants(), reaction.products()] {
                    if let Some(list) = list.get() {
                        identifiers.extend(list.iter().filter_map(|it| it.id().get()));
                    }
                }
            }
        }
        identifiers
    }

    /// Returns a vector of [FunctionDefinition] identifiers (attribute **id**).
    pub(crate) fn reaction_identifiers(&self) -> Vec<SId> {
        if let Some(reactions) = self.reactions().get() {
            reactions
                .iter()
                .map(|reaction| reaction.id().get())
                .collect::<Vec<SId>>()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of all *variables* appearing in all [AssignmentRule] objects.
    pub(crate) fn assignment_rule_variables(&self) -> Vec<SId> {
        if let Some(rules) = self.rules().get() {
            rules
                .iter()
                .filter_map(|rule| rule.try_downcast::<AssignmentRule>())
                .map(|assignment_rule| assignment_rule.variable().get())
                .collect::<Vec<SId>>()
        } else {
            Vec::new()
        }
    }

    /// Returns a vector of values from within the **ci** elements appearing in all [AlgebraicRule]
    /// objects in this model.
    ///
    /// Does not include instances when **ci** is used as an argument of the `rateOf` symbol, since
    /// this technically does not count as a "variable" (i.e. the expression determines the
    /// rate of the symbol, not the value of the symbol).  
    pub(crate) fn algebraic_rule_ci_variables(&self) -> Vec<SId> {
        if let Some(rules) = self.rules().get() {
            rules
                .iter()
                .filter_map(|rule| rule.try_downcast::<AlgebraicRule>())
                .filter_map(|algebraic_rule| algebraic_rule.math().get())
                .flat_map(|math| {
                    math.recursive_child_elements()
                        .into_iter()
                        .filter(|child| {
                            if child.tag_name() == "ci" {
                                if let Some(parent) = child.parent() {
                                    let is_apply = parent.tag_name() == "apply";
                                    let is_rate_of = parent
                                        .get_child_at(0)
                                        .map(|it| {
                                            it.get_attribute("definitionURL").is_some_and(|url| {
                                                url == "http://www.sbml.org/sbml/symbols/rateOf"
                                            })
                                        })
                                        .unwrap_or(false);
                                    !(is_apply && is_rate_of)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                        .map(|ci| ci.text_content())
                        .map(|text| {
                            // TODO: This should be a validation step (ci elements can only use SId compliant identifiers)
                            // SBML Core Specification, Section 3.4.3
                            SId::try_from(text)
                                .expect("The contents of <ci> must be an SBML identifier.")
                        })
                        .collect::<Vec<SId>>()
                })
                .collect::<Vec<SId>>()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn is_rateof_target_constant(&self, target: &str) -> bool {
        fn find_target<T: SBase>(target: &str, object: &XmlList<T>) -> Option<T> {
            for x in object.iter() {
                if let Some(value) = object.id().get() {
                    if value.as_str() == target {
                        return Some(x);
                    }
                }
            }
            None
        }

        let compartments = self.compartments().get();
        let parameters = self.parameters().get();
        let species = self.species().get();

        if let Some(compartment) = compartments.and_then(|list| find_target(target, &list)) {
            return compartment.constant().get();
        }
        if let Some(parameter) = parameters.and_then(|list| find_target(target, &list)) {
            return parameter.constant().get();
        }
        if let Some(species) = species.and_then(|list| find_target(target, &list)) {
            return species.constant().get();
        }

        if let Some(reactions) = self.reactions().get() {
            for reaction in reactions.iter() {
                let species_ref = reaction.reactants().get();
                let products = reaction.products().get();
                if let Some(species_ref) = species_ref.and_then(|list| find_target(target, &list)) {
                    return species_ref.constant().get();
                }
                if let Some(species_ref) = products.and_then(|list| find_target(target, &list)) {
                    return species_ref.constant().get();
                }
            }
        }
        true
    }

    /// Finds a species with the given *id*. If not found, returns `None`.
    pub(crate) fn find_species(&self, id: &str) -> Option<Species> {
        if let Some(species) = self.species().get() {
            species
                .iter()
                .find(|species| species.id().get().as_str() == id)
        } else {
            None
        }
    }

    /// Finds a compartment with the given *id*. If not found, returns `None`.
    pub(crate) fn find_compartment(&self, id: &str) -> Option<Compartment> {
        if let Some(compartments) = self.compartments().get() {
            compartments
                .iter()
                .find(|compartment| compartment.id().get().as_str() == id)
        } else {
            None
        }
    }
}
