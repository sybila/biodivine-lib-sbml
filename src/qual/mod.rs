mod model;
mod qualitative_species;
mod transition;
pub mod types;

pub use model::QualModel;
pub use qualitative_species::QualitativeSpecies;
pub use transition::{DefaultTerm, FunctionTerm, Input, ListOfFunctionTerms, Output, Transition};

#[cfg(test)]
mod tests {
    use crate::qual::QualModel;
    use crate::xml::OptionalXmlChild;
    use crate::Sbml;

    #[test]
    fn basic_qual_test() {
        let doc = Sbml::read_path("test-inputs/model.sbml").unwrap();
        let model = doc.model().get().unwrap();
        let model = QualModel::from_model(model);
        let species = model.list_of_qualitative_species().get().unwrap();
        assert_eq!(4, species.len());
        let transitions = model.list_of_transitions().get().unwrap();
        assert_eq!(4, transitions.len());

        /*
        TODO: This is not working because attributes with namespaces are broken.
        let species_p53 = species.get(0);
        assert_eq!("p53", species_p53.id().get());
        assert_eq!(2, species_p53.max_level().get().unwrap());
        assert_eq!(false, species_p53.constant().get());
        assert_eq!(None, species_p53.name().get());

        let species_mdm2cyt = species.get(1);
        assert_eq!("Mdm2cyt", species_mdm2cyt.id().get());
        assert_eq!(2, species_p53.max_level().get().unwrap());
        assert_eq!(false, species_p53.constant().get());
        assert_eq!(None, species_p53.name().get());

        let species_mdm2nuc = species.get(2);
        assert_eq!("Mdm2nuc", species_mdm2nuc.id().get());
        assert_eq!(1, species_mdm2nuc.max_level().get().unwrap());
        assert_eq!(false, species_mdm2nuc.constant().get());
        assert_eq!(None, species_mdm2nuc.name().get());

        let species_dna_dam = species.get(3);
        assert_eq!("DNAdam", species_dna_dam.id().get());
        assert_eq!(1, species_dna_dam.max_level().get().unwrap());
        assert_eq!(false, species_dna_dam.constant().get());
        assert_eq!(None, species_dna_dam.name().get());
         */
    }
}
