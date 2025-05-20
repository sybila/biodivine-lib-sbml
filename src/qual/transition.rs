use crate::constants::namespaces::NS_QUAL;
use crate::core::sbase::SbmlUtils;
use crate::qual::qual_input::QualInput;
use crate::qual::qual_output::QualOutput;
use crate::qual::terms::AbstractTerm;
use crate::xml::{OptionalChild, OptionalXmlChild, RequiredChild, RequiredXmlChild, XmlDocument, XmlElement, XmlList};
use sbml_macros::{SBase, XmlWrapper};

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Transition(XmlElement);

impl Transition {
    pub fn new(document: XmlDocument, function_terms: XmlList<AbstractTerm>) -> Transition {
        let obj = Transition::new_empty(document, "transition");
        obj.function_terms().set(function_terms);
        obj
    }
    pub fn inputs(&self) -> OptionalChild<XmlList<QualInput>> {
        self.optional_package_child("listOfInputs", NS_QUAL, true)
    }

    pub fn outputs(&self) -> OptionalChild<XmlList<QualOutput>> {
        self.optional_package_child("listOfOutputs", NS_QUAL, true)
    }

    pub fn function_terms(&self) -> RequiredChild<XmlList<AbstractTerm>> {
        self.required_package_child("listOfFunctionTerms", NS_QUAL, true)
    }
}


pub fn get_outputs_from_transition(transition: Transition) -> Vec<QualOutput> {
    let mut lst = Vec::new();

    if !transition.outputs().is_set(){
        return lst
    }

    for output in transition.outputs() {
        lst.push(output);
    }

    lst
}