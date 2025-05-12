mod objective;
mod flux_objective;
mod gene_product;
mod gene_product_association;
mod association;
mod fbc_species;
mod fbc_reaction;


pub use objective::Objective;
pub use gene_product::GeneProduct;
use crate::constraint::association::GeneProductRef;
use crate::constraint::fbc_reaction::FbcReaction;
use crate::Sbml;
use crate::core::sbase::SbmlUtils;
use crate::core::{SId};
use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlSupertype};

#[test]
pub fn basic_test() {
    let doc = Sbml::read_path("test-inputs/fbc_tests/example_fbc.xml");
    let reactions = doc.unwrap().model().get().unwrap().reactions().get().unwrap();
    
    let r1: FbcReaction = reactions.get(0).try_downcast().unwrap();
    let r2: FbcReaction = reactions.get(1).try_downcast().unwrap();
    
    let gene_product_ref1 = r1.gene_product_association().get().unwrap().gene_product_ref().get().unwrap(); 
    let and2 = r2.gene_product_association().get().unwrap().and().get().unwrap();
   
    assert_eq!(gene_product_ref1.geneProduct().get(), SId::try_from("g1").unwrap());
    
    let gene_product2_1: GeneProductRef = and2.get(0).try_downcast().unwrap();
    let gene_product2_2: GeneProductRef = and2.get(1).try_downcast().unwrap();
    
    assert_eq!(gene_product2_1.geneProduct().get(), SId::try_from("g1").unwrap());
    assert_eq!(gene_product2_2.geneProduct().get(), SId::try_from("g4").unwrap());
    
}