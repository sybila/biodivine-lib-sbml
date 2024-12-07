use crate::core::sbase::SbmlUtils;
use crate::core::Model;
use crate::qual::transition::Transition;
use crate::qual::QualitativeSpecies;
use crate::xml::{OptionalChild, XmlDefault, XmlDocument, XmlElement, XmlList, XmlWrapper};
use sbml_macros::{SBase, XmlWrapper};

// TODO:
//  We need to figure out how to properly "cast" the qual model. In particular, it would
//  be nice to have a mechanism which allows us to specify at compile time which extensions
//  should be available and then make those explicitly available.
//
//  The problem with this is that every SBML element would have to carry some kind of
//  "enabled extensions" type argument, which is super super cumbersome.
//
//  The other options is to just do runtime checks and have plugin specific "views" of the `Sbml`
//  object so that we can access the extra elements safely. This is probably the way we should go.
//

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct QualModel(XmlElement);

impl XmlDefault for QualModel {
    fn default(document: XmlDocument) -> Self {
        QualModel::new_empty(document, "model")
    }
}

impl QualModel {
    pub fn from_model(model: Model) -> QualModel {
        // TODO:
        //  We need a better way to do this once we figure out how to work with plugins.
        unsafe { Self::unchecked_cast(model) }
    }

    pub fn list_of_transitions(&self) -> OptionalChild<XmlList<Transition>> {
        self.optional_qual_child("listOfTransitions")
    }

    pub fn list_of_qualitative_species(&self) -> OptionalChild<XmlList<QualitativeSpecies>> {
        self.optional_qual_child("listOfQualitativeSpecies")
    }
}
