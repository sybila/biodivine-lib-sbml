mod association;
mod fbc_reaction;
mod fbc_species;
mod flux_objective;
mod gene_product;
mod gene_product_association;
mod objective;
mod validation;

pub use fbc_reaction::FbcReaction;
pub use fbc_species::FbcSpecies;
pub use gene_product::GeneProduct;
pub use objective::FbcType;
pub use objective::Objective;

#[cfg(test)]
mod tests {
    use crate::constraint::association::GeneProductRef;
    use crate::constraint::FbcReaction;
    use crate::core::SId;
    use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlSupertype};
    use crate::Sbml;

    #[test]
    pub fn basic_test() {
        let doc = Sbml::read_path("test-inputs/test-fbc/example_fbc.xml");
        assert!(doc.clone().unwrap().validate().is_empty());

        let reactions = doc
            .unwrap()
            .model()
            .get()
            .unwrap()
            .reactions()
            .get()
            .unwrap();

        let r1: FbcReaction = reactions.get(0).try_downcast().unwrap();
        let r2: FbcReaction = reactions.get(1).try_downcast().unwrap();

        let gene_product_ref1 = r1
            .gene_product_association()
            .get()
            .unwrap()
            .gene_product_ref()
            .get()
            .unwrap();
        let and2 = r2
            .gene_product_association()
            .get()
            .unwrap()
            .and()
            .get()
            .unwrap();

        assert_eq!(
            gene_product_ref1.gene_product().get(),
            SId::try_from("g1").unwrap()
        );

        let gene_product2_1: GeneProductRef = and2.get(0).try_downcast().unwrap();
        let gene_product2_2: GeneProductRef = and2.get(1).try_downcast().unwrap();

        assert_eq!(
            gene_product2_1.gene_product().get(),
            SId::try_from("g1").unwrap()
        );
        assert_eq!(
            gene_product2_2.gene_product().get(),
            SId::try_from("g4").unwrap()
        );
    }
}
