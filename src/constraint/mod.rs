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
use crate::tests::sid;
use crate::xml::{OptionalXmlChild, RequiredXmlProperty, XmlSupertype};

#[test]
pub fn test() {
    let doc = Sbml::read_path("test-inputs/fbc_tests/example_fbc.xml");
    let reactions = doc.unwrap().model().get().unwrap().reactions().get().unwrap();
    
    let r1: FbcReaction = reactions.get(0).try_downcast().unwrap();
    let r2: FbcReaction = reactions.get(1).try_downcast().unwrap();
    
    let geneProductRef1 = r1.geneProductAssociation().get().unwrap().geneProductRef().get().unwrap(); 
    let and2 = r2.geneProductAssociation().get().unwrap().and().get().unwrap();
   
    assert_eq!(geneProductRef1.geneProduct().get(), sid("g1"));
    
    let geneProduct2_1: GeneProductRef = and2.get(0).try_downcast().unwrap();
    let geneProduct2_2: GeneProductRef = and2.get(1).try_downcast().unwrap();
    
    assert_eq!(geneProduct2_1.geneProduct().get(), sid("g1"));
    assert_eq!(geneProduct2_2.geneProduct().get(), sid("g4"));
    
}