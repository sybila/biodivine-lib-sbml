use crate::constants::namespaces::NS_FBC;
use crate::core::sbase::SbmlUtils;
use crate::core::SId;
use crate::xml::{
    Document, OptionalSbmlProperty, RequiredSbmlProperty, RequiredXmlProperty, XmlElement,
};
use sbml_macros::{SBase, XmlWrapper};

/// The `<fluxObjective>` element of the FBC (Flux Balance Constraints) package specification.
#[derive(Clone, Debug, SBase, XmlWrapper)]
#[sbml_object(name = "fluxObjective", namespace = NS_FBC)]
pub struct FluxObjective(XmlElement);

impl FluxObjective {
    /// Creates a new `<fluxObjective>` element with the given `reaction`, `coefficient`.
    pub fn new(document: Document, reaction: SId, coefficient: f64) -> FluxObjective {
        let obj = FluxObjective::new_empty(document);
        obj.reaction().set(&reaction);
        obj.coefficient().set(&coefficient);
        obj
    }
    /// The `id` attribute of this `<fluxObjective>` element.
    pub fn id(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_package_property("id", NS_FBC, NS_FBC)
    }
    /// The `name` attribute of this `<fluxObjective>` element.
    pub fn name(&self) -> OptionalSbmlProperty<'_, String> {
        self.optional_package_property("name", NS_FBC, NS_FBC)
    }
    /// The `reaction` attribute of this `<fluxObjective>` element.
    pub fn reaction(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_package_property("reaction", NS_FBC, NS_FBC)
    }
    /// The `coefficient` attribute of this `<fluxObjective>` element.
    pub fn coefficient(&self) -> RequiredSbmlProperty<'_, f64> {
        self.required_package_property("coefficient", NS_FBC, NS_FBC)
    }
}
