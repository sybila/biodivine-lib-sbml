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
pub struct QualOutput(XmlElement);

impl QualOutput {
    pub fn new(
        document: XmlDocument,
        qualitative_species: SId,
        transition_effect: TransitionOutputEffect,
    ) -> QualOutput {
        let obj = QualOutput::new_empty(document, "output");
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

    pub fn qualitative_species(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("qualitativeSpecies", NS_QUAL, NS_QUAL)
    }

    pub fn transition_effect(&self) -> RequiredSbmlProperty<TransitionOutputEffect> {
        self.required_package_property("transitionEffect", NS_QUAL, NS_QUAL)
    }

    pub fn output_level(&self) -> OptionalSbmlProperty<u32> {
        self.optional_package_property("outputLevel", NS_QUAL, NS_QUAL)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionOutputEffect {
    Production,
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
