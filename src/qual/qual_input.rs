use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument, XmlElement,
    XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct QualInput(XmlElement);

impl QualInput {
    pub fn new(
        document: XmlDocument,
        qualitative_species: SId,
        transition_effect: TransitionInputEffect,
    ) -> QualInput {
        let obj = QualInput::new_empty(document, "input");
        obj.qualitative_species().set(&qualitative_species);
        obj.transition_effect().set(&transition_effect);
        obj
    }
    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_QUAL, NS_QUAL)
    }

    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_QUAL, NS_QUAL)
    }

    pub fn sign(&self) -> OptionalSbmlProperty<Sign> {
        self.optional_package_property("sign", NS_QUAL, NS_QUAL)
    }

    pub fn qualitative_species(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("qualitativeSpecies", NS_QUAL, NS_QUAL)
    }

    pub fn transition_effect(&self) -> RequiredSbmlProperty<TransitionInputEffect> {
        self.required_package_property("transitionEffect", NS_QUAL, NS_QUAL)
    }

    pub fn threshold_level(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("thresholdLevel", NS_QUAL, NS_QUAL)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
    Dual,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionInputEffect {
    None,
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
