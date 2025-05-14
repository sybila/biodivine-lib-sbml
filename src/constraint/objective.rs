use crate::constants::namespaces::NS_FBC;
use crate::constraint::flux_objective::FluxObjective;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    OptionalSbmlProperty, RequiredChild, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument,
    XmlElement, XmlList, XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FbcType {
    Maximize,
    Minimize,
}

impl Display for FbcType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let xsi = match self {
            FbcType::Maximize => "maximize",
            FbcType::Minimize => "minimize",
        };

        write!(f, "{}", xsi)
    }
}

impl From<FbcType> for String {
    fn from(value: FbcType) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for FbcType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "maximize" => Ok(FbcType::Maximize),
            "minimize" => Ok(FbcType::Minimize),
            _ => Err(format!("FbcType '{value}' is not valid.")),
        }
    }
}

impl XmlPropertyType for FbcType {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match FbcType::try_from(value.to_string()) {
                Ok(xsi) => Ok(Some(xsi)),
                Err(_) => Err(format!("FbcType '{value}' is not valid.")),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}
#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct Objective(XmlElement);

impl Objective {
    pub fn new(document: XmlDocument, id: SId, bound_type: FbcType) -> Objective {
        let obj = Objective::new_empty(document, "objective");
        obj.id().set(&id);
        obj.bound_type().set(&bound_type);
        obj
    }
    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn bound_type(&self) -> RequiredSbmlProperty<FbcType> {
        self.required_package_property("type", NS_FBC, NS_FBC)
    }

    pub fn flux_objectives(&self) -> RequiredChild<XmlList<FluxObjective>> {
        self.required_package_child("listOfFluxObjectives", NS_FBC, false)
    }
}
