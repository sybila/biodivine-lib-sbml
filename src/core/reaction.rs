use crate::constants::namespaces::NS_SBML_CORE;
use crate::core::sbase::SbmlUtils;
use crate::core::{SBase, SId};
use crate::math::{Math, MathNode};
use crate::xml::{
    Document, OptionalChild, OptionalSbmlChild, OptionalSbmlProperty, OptionalXmlChild,
    RequiredSbmlProperty, RequiredXmlProperty, XmlDefault, XmlElement, XmlList, XmlProperty,
    XmlSupertype,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "reaction", namespace = NS_SBML_CORE)]
pub struct Reaction(XmlElement);

impl XmlSupertype for Reaction {}
impl Reaction {
    pub fn new(document: Document, id: &SId, reversible: bool) -> Self {
        let obj = Reaction::new_empty(document);
        obj.id().set(id);
        obj.reversible().set(&reversible);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("id")
    }

    pub fn reversible(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("reversible")
    }

    pub fn compartment(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("compartment")
    }

    pub fn reactants(&self) -> OptionalSbmlChild<'_, XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfReactants")
    }

    pub fn products(&self) -> OptionalSbmlChild<'_, XmlList<SpeciesReference>> {
        self.optional_sbml_child("listOfProducts")
    }

    pub fn modifiers(&self) -> OptionalSbmlChild<'_, XmlList<ModifierSpeciesReference>> {
        self.optional_sbml_child("listOfModifiers")
    }

    pub fn kinetic_law(&self) -> OptionalSbmlChild<'_, KineticLaw> {
        self.optional_sbml_child("kineticLaw")
    }
}

pub trait SimpleSpeciesReference: SBase {
    fn species(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("species")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "speciesReference", namespace = NS_SBML_CORE)]
pub struct SpeciesReference(XmlElement);

impl SimpleSpeciesReference for SpeciesReference {}

impl SpeciesReference {
    pub fn new(document: Document, species: &SId, constant: bool) -> Self {
        let obj = SpeciesReference::new_empty(document);
        obj.species().set(species);
        obj.constant().set(&constant);
        obj
    }

    pub fn stoichiometry(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("stoichiometry")
    }

    pub fn constant(&self) -> RequiredSbmlProperty<'_, bool> {
        self.required_sbml_property("constant")
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "modifierSpeciesReference", namespace = NS_SBML_CORE)]
pub struct ModifierSpeciesReference(XmlElement);

impl SimpleSpeciesReference for ModifierSpeciesReference {}

impl ModifierSpeciesReference {
    pub fn new(document: Document, species: &SId) -> Self {
        let obj = ModifierSpeciesReference::new_empty(document);
        obj.species().set(species);
        obj
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "kineticLaw", namespace = NS_SBML_CORE)]
pub struct KineticLaw(XmlElement);

impl XmlDefault for KineticLaw {
    fn default(document: Document) -> Self {
        KineticLaw::new_empty(document)
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

    pub fn math(&self) -> OptionalChild<'_, Math> {
        self.optional_math_child("math")
    }

    /// The MathML formula of this object as a typed node, or `None` when it has no `math` child.
    ///
    /// [`math`](Self::math) gives the raw element instead; attaching a new formula to a site is
    /// `self.math().set(Math::new(document, node))`.
    pub fn math_node(&self) -> Option<MathNode> {
        self.math().get().map(|it| it.node())
    }

    pub fn local_parameters(&self) -> OptionalSbmlChild<'_, XmlList<LocalParameter>> {
        self.optional_sbml_child("listOfLocalParameters")
    }

    pub(crate) fn local_parameter_identifiers(&self) -> Vec<SId> {
        if let Some(local_parameters) = self.local_parameters().get() {
            local_parameters
                .iter()
                .filter_map(|param| XmlProperty::get_checked(&param.id()).ok().flatten())
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, XmlWrapper, SBase)]
#[sbml_object(name = "localParameter", namespace = NS_SBML_CORE)]
pub struct LocalParameter(XmlElement);

impl LocalParameter {
    pub fn new(document: Document, id: &SId) -> Self {
        let obj = LocalParameter::new_empty(document);
        obj.id().set(id);
        obj
    }

    pub fn id(&self) -> RequiredSbmlProperty<'_, SId> {
        self.required_sbml_property("id")
    }

    pub fn value(&self) -> OptionalSbmlProperty<'_, f64> {
        self.optional_sbml_property("value")
    }

    pub fn units(&self) -> OptionalSbmlProperty<'_, SId> {
        self.optional_sbml_property("units")
    }
}
