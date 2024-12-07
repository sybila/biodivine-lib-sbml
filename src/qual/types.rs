use crate::xml::XmlPropertyType;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Copy, Clone, PartialEq, Eq, EnumString, Display)]
pub enum Sign {
    #[strum(serialize = "positive")]
    Positive,
    #[strum(serialize = "negative")]
    Negative,
    #[strum(serialize = "dual")]
    Dual,
    #[strum(serialize = "unknown")]
    Unknown,
}

#[derive(Copy, Clone, PartialEq, Eq, EnumString, Display)]
pub enum TransitionInputEffect {
    #[strum(serialize = "consumption")]
    Consumption,
    #[strum(serialize = "none")]
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, EnumString, Display)]
pub enum TransitionOutputEffect {
    #[strum(serialize = "production")]
    Production,
    #[strum(serialize = "assignmentLevel")]
    AssignmentLevel,
}

impl XmlPropertyType for Sign {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match Sign::from_str(value) {
                Ok(unit) => Ok(Some(unit)),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid sign ({})",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl XmlPropertyType for TransitionInputEffect {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match TransitionInputEffect::from_str(value) {
                Ok(unit) => Ok(Some(unit)),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid transition input effect ({})",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl XmlPropertyType for TransitionOutputEffect {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match TransitionOutputEffect::from_str(value) {
                Ok(unit) => Ok(Some(unit)),
                Err(e) => Err(format!(
                    "Value `{value}` does not represent a valid transition output effect ({})",
                    e
                )),
            },
            None => Err("Value missing".to_string()),
        }
    }

    fn set(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}
