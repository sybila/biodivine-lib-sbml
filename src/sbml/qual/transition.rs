use crate::sbase::SbmlUtils;
use crate::sbml::qual::types::{Sign, TransitionInputEffect, TransitionOutputEffect};
use crate::xml::{
    OptionalChild, OptionalProperty, RequiredChild, RequiredProperty, XmlElement, XmlList,
    XmlWrapper,
};
use macros::{SBase, XmlWrapper};
use std::ops::Deref;

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Transition(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Input(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct Output(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ListOfFunctionTerms(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct DefaultTerm(XmlElement);

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct FunctionTerm(XmlElement);

impl Transition {
    pub fn id(&self) -> OptionalProperty<String> {
        self.optional_qual_property("id")
    }

    pub fn name(&self) -> OptionalProperty<String> {
        self.optional_qual_property("name")
    }

    pub fn list_of_inputs(&self) -> OptionalChild<XmlList<Input>> {
        self.optional_qual_child("listOfInputs")
    }

    pub fn list_of_outputs(&self) -> OptionalChild<XmlList<Output>> {
        self.optional_qual_child("listOfOutputs")
    }

    pub fn list_of_function_terms(&self) -> OptionalChild<ListOfFunctionTerms> {
        self.optional_qual_child("listOfFunctionTerms")
    }
}

impl Input {
    pub fn id(&self) -> OptionalProperty<String> {
        self.optional_qual_property("id")
    }

    pub fn name(&self) -> OptionalProperty<String> {
        self.optional_qual_property("name")
    }

    pub fn sign(&self) -> OptionalProperty<Sign> {
        self.optional_qual_property("sign")
    }

    pub fn qualitative_species(&self) -> RequiredProperty<String> {
        self.required_qual_property("qualitativeSpecies")
    }

    pub fn transition_effect(&self) -> RequiredProperty<TransitionInputEffect> {
        self.required_qual_property("transitionEffect")
    }

    pub fn threshold_level(&self) -> OptionalProperty<u32> {
        self.optional_qual_property("thresholdLevel")
    }
}

impl Output {
    pub fn id(&self) -> OptionalProperty<String> {
        self.optional_qual_property("id")
    }

    pub fn name(&self) -> OptionalProperty<String> {
        self.optional_qual_property("name")
    }

    pub fn qualitative_species(&self) -> RequiredProperty<String> {
        self.required_qual_property("qualitativeSpecies")
    }

    pub fn transition_effect(&self) -> RequiredProperty<TransitionOutputEffect> {
        self.required_qual_property("transitionEffect")
    }

    pub fn output_level(&self) -> OptionalProperty<u32> {
        self.optional_qual_property("outputLevel")
    }
}

impl DefaultTerm {
    pub fn result_level(&self) -> RequiredProperty<u32> {
        self.required_qual_property("resultLevel")
    }
}

impl FunctionTerm {
    pub fn result_level(&self) -> RequiredProperty<u32> {
        self.required_qual_property("resultLevel")
    }

    pub fn math(&self) -> RequiredChild<XmlElement> {
        self.required_math_child("math")
    }
}

impl ListOfFunctionTerms {
    pub fn default_term(&self) -> RequiredChild<DefaultTerm> {
        self.required_math_child("defaultTerm")
    }

    pub fn len_function_term(&self) -> usize {
        let doc = self.read_doc();
        // -1 is for the default term
        self.raw_element().child_elements(doc.deref()).len() - 1
    }

    pub fn get_function_term(&self, index: usize) -> FunctionTerm {
        self.get_function_term_checked(index)
            .unwrap_or_else(|| panic!("No XML element at position {index}."))
    }

    pub fn get_function_term_checked(&self, index: usize) -> Option<FunctionTerm> {
        let doc = self.read_doc();
        let children = self.raw_element().children(doc.deref());
        children
            .iter()
            .map(|it| {
                it.as_element().unwrap_or_else(|| {
                    panic!("Item at position {index} is not an XML element.");
                })
            })
            .filter(|it| it.name(doc.deref()) == "FunctionTerm")
            .nth(index)
            .map(|it| FunctionTerm(XmlElement::new_raw(self.document(), it)))
    }

    // TODO: Add methods for updating the term list.
}
