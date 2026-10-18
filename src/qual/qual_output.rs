use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
    XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

/// The `<output>` element of the Qual (Qualitative Models) package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "output", namespace = NS_QUAL)]
pub struct QualOutput(XmlElement);

impl QualOutput {
    /// Creates a new `<output>` element with the given `qualitative_species`, `transition_effect`.
    pub fn new(
        document: Document,
        qualitative_species: SId,
        transition_effect: TransitionOutputEffect,
    ) -> QualOutput {
        let obj = QualOutput::new_empty(document);
        obj.qualitative_species().set(&qualitative_species);
        obj.transition_effect().set(&transition_effect);
        obj
    }
    /// The `id` attribute of this `<output>` element.
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_QUAL, NS_QUAL)
    }

    /// The `name` attribute of this `<output>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_QUAL, NS_QUAL)
    }

    /// The `qualitativeSpecies` attribute of this `<output>` element.
    pub fn qualitative_species(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("qualitativeSpecies", NS_QUAL, NS_QUAL)
    }

    /// The `transitionEffect` attribute of this `<output>` element.
    pub fn transition_effect(&self) -> RequiredSbmlProperty<'_, TransitionOutputEffect> {
        self.required_package_property("transitionEffect", NS_QUAL, NS_QUAL)
    }

    /// The `outputLevel` attribute of this `<output>` element.
    pub fn output_level(&self) -> OptionalSbmlProperty<'_, u32> {
        self.optional_package_property("outputLevel", NS_QUAL, NS_QUAL)
    }
}

/// The `qual:transitionEffect` of an output (Qual Package, Section 3.3): what the transition does
/// to the species the output refers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionOutputEffect {
    /// The transition produces the species, i.e. raises its level.
    Production,
    /// The transition sets the level of the species to the value the output gives.
    AssignmentLevel,
}

impl TryFrom<String> for TransitionOutputEffect {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "production" => Ok(TransitionOutputEffect::Production),
            "assignmentlevel" => Ok(TransitionOutputEffect::AssignmentLevel),
            _ => Err(format!("'{value}' is not valid TransitionInputEffect type")),
        }
    }
}

impl Display for TransitionOutputEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            TransitionOutputEffect::Production => "production",
            TransitionOutputEffect::AssignmentLevel => "assignmentLevel",
        };

        write!(f, "{}", str)
    }
}

impl XmlPropertyType for TransitionOutputEffect {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match TransitionOutputEffect::try_from(value.to_string()) {
                Ok(output_effect) => Ok(Some(output_effect)),
                Err(message) => Err(message),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}
