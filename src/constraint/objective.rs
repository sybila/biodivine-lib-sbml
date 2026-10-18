use crate::constants::namespaces::NS_FBC;
use crate::constraint::flux_objective::FluxObjective;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlChild, RequiredSbmlProperty, RequiredXmlProperty,
    XmlElement, XmlList, XmlPropertyType,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;
/// The `fbc:type` of an objective (FBC Package, Section 3.3): the direction in which the
/// objective is optimized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FbcType {
    /// The objective is maximized.
    Maximize,
    /// The objective is minimized.
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
            _ => Err(format!("FbcType '{value}' is not valid value.")),
        }
    }
}

impl XmlPropertyType for FbcType {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match FbcType::try_from(value.to_string()) {
                Ok(fbc) => Ok(Some(fbc)),
                Err(message) => Err(message),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}
/// The `<objective>` element of the FBC (Flux Balance Constraints) package specification.
#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "objective", namespace = NS_FBC)]
pub struct Objective(XmlElement);

impl Objective {
    /// Creates a new `<objective>` element with the given `id`, `bound_type`.
    pub fn new(document: Document, id: SId, bound_type: FbcType) -> Objective {
        let obj = Objective::new_empty(document);
        obj.id().set(&id);
        obj.bound_type().set(&bound_type);
        obj
    }
    /// The `id` attribute of this `<objective>` element.
    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("id", NS_FBC, NS_FBC)
    }
    /// The `name` attribute of this `<objective>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    /// The `type` attribute of this `<objective>` element.
    pub fn bound_type(&self) -> RequiredSbmlProperty<'_, FbcType> {
        self.required_package_property("type", NS_FBC, NS_FBC)
    }

    /// The `listOfFluxObjectives` child of this `<objective>` element, as a list of [`FluxObjective`].
    pub fn flux_objectives(&self) -> RequiredSbmlChild<'_, XmlList<FluxObjective>> {
        self.required_package_child("listOfFluxObjectives", NS_FBC, false)
    }
}
