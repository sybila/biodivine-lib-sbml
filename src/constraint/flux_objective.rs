use sbml_macros::{SBase, XmlWrapper};
use crate::constants::namespaces::NS_FBC;
use crate::constraint::Objective;
use crate::constraint::objective::FbcType;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlDocument, XmlElement};

#[derive(Clone, Debug, SBase, XmlWrapper)]
pub struct FluxObjective(XmlElement);

impl FluxObjective {
    pub fn new(document: XmlDocument, reaction: SId, coefficient: f64) -> FluxObjective {
        let obj = FluxObjective::new_empty(document, "fluxObjective");
        obj.reaction().set(&reaction);
        obj.coefficient().set(&coefficient);
        obj
    }
    pub fn id(&self) -> OptionalSbmlProperty<SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    pub fn name(&self) -> OptionalSbmlProperty<String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    pub fn reaction(&self) -> RequiredSbmlProperty<SId> {
        self.required_package_property("reaction", NS_FBC, NS_FBC)
    }
    pub fn coefficient(&self) -> RequiredSbmlProperty<f64> {
        self.required_package_property("coefficient", NS_FBC, NS_FBC)
    }
}