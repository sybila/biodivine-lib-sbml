use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
    XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

/// The `<input>` element of the Qual (Qualitative Models) package specification.
#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "input", namespace = NS_QUAL)]
pub struct QualInput(XmlElement);

impl QualInput {
    /// Creates a new `<input>` element with the given `qualitative_species`, `transition_effect`.
    pub fn new(
        document: Document,
        qualitative_species: SId,
        transition_effect: TransitionInputEffect,
    ) -> QualInput {
        let obj = QualInput::new_empty(document);
        obj.qualitative_species().set(&qualitative_species);
        obj.transition_effect().set(&transition_effect);
        obj
    }
    /// The `id` attribute of this `<input>` element.
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_QUAL, NS_QUAL)
    }

    /// The `name` attribute of this `<input>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_QUAL, NS_QUAL)
    }

    /// The `sign` attribute of this `<input>` element.
    pub fn sign(&self) -> OptionalSbmlProperty<'_, Sign> {
        self.optional_package_property("sign", NS_QUAL, NS_QUAL)
    }

    /// The `qualitativeSpecies` attribute of this `<input>` element.
    pub fn qualitative_species(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("qualitativeSpecies", NS_QUAL, NS_QUAL)
    }

    /// The `transitionEffect` attribute of this `<input>` element.
    pub fn transition_effect(&self) -> RequiredSbmlProperty<'_, TransitionInputEffect> {
        self.required_package_property("transitionEffect", NS_QUAL, NS_QUAL)
    }

    /// The `thresholdLevel` attribute of this `<input>` element.
    pub fn threshold_level(&self) -> OptionalSbmlProperty<'_, u32> {
        self.optional_package_property("thresholdLevel", NS_QUAL, NS_QUAL)
    }
}

/// The `qual:sign` of an input (Qual Package, Section 3.3): the direction in which the input
/// influences its transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    /// The input increases the level of its qualitative species.
    Positive,
    /// The input decreases the level of its qualitative species.
    Negative,
    /// The input can both increase and decrease the level of its qualitative species.
    Dual,
    /// The direction of the influence is unknown.
    Unknown,
}

impl TryFrom<String> for Sign {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "positive" => Ok(Sign::Positive),
            "negative" => Ok(Sign::Negative),
            "dual" => Ok(Sign::Dual),
            "unknown" => Ok(Sign::Unknown),
            _ => Err(format!("Sign '{value}' is not valid value of sign type")),
        }
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Sign::Positive => "positive",
            Sign::Negative => "negative",
            Sign::Dual => "dual",
            Sign::Unknown => "unknown",
        };

        write!(f, "{}", str)
    }
}

impl XmlPropertyType for Sign {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match Sign::try_from(value.to_string()) {
                Ok(sign) => Ok(Some(sign)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}

/// The `qual:transitionEffect` of an input (Qual Package, Section 3.3): what the transition does
/// to the species the input refers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionInputEffect {
    /// The transition does not change the level of the species.
    None,
    /// The transition consumes the species, i.e. lowers its level.
    Consumption,
}

impl TryFrom<String> for TransitionInputEffect {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "none" => Ok(TransitionInputEffect::None),
            "consumption" => Ok(TransitionInputEffect::Consumption),
            _ => Err(format!("'{value}' is not valid TransitionInputEffect type")),
        }
    }
}

impl Display for TransitionInputEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            TransitionInputEffect::None => "none",
            TransitionInputEffect::Consumption => "consumption",
        };

        write!(f, "{}", str)
    }
}

impl XmlPropertyType for TransitionInputEffect {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match TransitionInputEffect::try_from(value.to_string()) {
                Ok(input_effect) => Ok(Some(input_effect)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}
