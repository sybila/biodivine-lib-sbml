use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::qual::qual_input::QualInput;
use crate::qual::qual_output::QualOutput;
use crate::qual::terms::AbstractTerm;
use crate::xml::{
    OptionalSbmlChild, OptionalXmlChild, RequiredSbmlChild, RequiredXmlChild, XmlDocument,
    XmlElement, XmlList,
};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Transition(XmlElement);

impl Transition {
    pub fn new(document: XmlDocument, function_terms: XmlList<AbstractTerm>) -> Transition {
        let obj = Transition::new_empty(document, "transition");
        obj.function_terms().set(function_terms);
        obj
    }
    pub fn inputs(&self) -> OptionalSbmlChild<XmlList<QualInput>> {
        self.optional_package_child("listOfInputs", NS_QUAL, true)
    }

    pub fn outputs(&self) -> OptionalSbmlChild<XmlList<QualOutput>> {
        self.optional_package_child("listOfOutputs", NS_QUAL, true)
    }

    pub fn function_terms(&self) -> RequiredSbmlChild<XmlList<AbstractTerm>> {
        self.required_package_child("listOfFunctionTerms", NS_QUAL, true)
    }
}

pub fn get_outputs_from_transition(transition: Transition) -> Vec<QualOutput> {
    let mut lst = Vec::new();

    if !transition.outputs().is_set() {
        return lst;
    }

    for i in 0..transition.outputs().get().unwrap().len() {
        lst.push(transition.outputs().get().unwrap().get(i));
    }

    lst
}
