use crate::core::sbase::SbmlUtils;
use crate::core::{Math, SBase, SId};
use crate::xml::{
    OptionalChild, OptionalSbmlProperty, OptionalXmlChild,
    RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlDocument, XmlElement, XmlList,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Reaction(XmlElement);

impl Reaction {
    pub fn new(document: XmlDocument, id: &SId, reversible: bool) -> Self {
        let obj = Reaction::new_empty(document, "reaction");
        obj.id().set(id);
        obj.reversible().set(&reversible);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("id")
    }

    pub fn reversible(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("reversible")
    }

    pub fn compartment(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("compartment")
    }

    pub fn reactants(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfReactants")
    }

    pub fn products(&self) -> OptionalChild<XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfProducts")
    }

    pub fn modifiers(&self) -> OptionalChild<XmlList<ModifierSpeciesReference>> {
        self.optional_sbml_child("listOfModifiers")
    }

    pub fn kinetic_law(&self) -> OptionalChild<KineticLaw> {
        self.optional_sbml_child("kineticLaw")
    }
}

pub trait SimpleSpeciesReference: SBase {
    fn species(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct SpeciesReference(XmlElement);

impl SimpleSpeciesReference for SpeciesReference {}

impl SpeciesReference {
    pub fn new(document: XmlDocument, species: &SId, constant: bool) -> Self {
        let obj = SpeciesReference::new_empty(document, "speciesReference");
        obj.species().set(species);
        obj.constant().set(&constant);
        obj
    }

    pub fn stoichiometry(&self) -> OptionalSbmlProperty<f64> {
        self.optional_sbml_property("stoichiometry")
    }

    pub fn constant(&self) -> RequiredSbmlProperty<bool> {
        self.required_sbml_property("constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ModifierSpeciesReference(XmlElement);

impl SimpleSpeciesReference for ModifierSpeciesReference {}

impl ModifierSpeciesReference {
    pub fn new(document: XmlDocument, species: &SId) -> Self {
        let obj = ModifierSpeciesReference::new_empty(document, "modifierSpeciesReference");
        obj.species().set(species);
        obj
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct KineticLaw(XmlElement);

impl XmlDefault for KineticLaw {
    fn default(document: XmlDocument) -> Self {
        KineticLaw::new_empty(document, "kineticLaw")
    }
}

impl KineticLaw {
    /// Try to find an instance of a [KineticLaw] element that is a parent of the given
    /// child element.
    ///
    /// The child can be any SBML tag, as long as one of its transitive parents is a
    /// [KineticLaw] element. If this is not satisfied, the method returns `None`.
    pub fn for_child_element(child: &XmlElement) -> Option<Self> {
        Self::search_in_parents(child, "kineticLaw")
    }

    pub fn math(&self) -> OptionalChild<Math> {
        self.optional_math_child("math")
    }

    pub fn local_parameters(&self) -> OptionalChild<XmlList<LocalParameter>> {
        self.optional_sbml_child("listOfLocalParameters")
    }

    pub(crate) fn local_parameter_identifiers(&self) -> Vec<SId> {
        if let Some(local_parameters) = self.local_parameters().get() {
            local_parameters
                .iter()
                .map(|param| param.id().get())
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct LocalParameter(XmlElement);

impl LocalParameter {
    pub fn new(document: XmlDocument, id: &SId) -> Self {
        let obj = LocalParameter::new_empty(document, "localParameter");
        obj.id().set(id);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<SId> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalSbmlProperty<f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalSbmlProperty<SId> {
        self.optional_sbml_property("units")
    }
}
