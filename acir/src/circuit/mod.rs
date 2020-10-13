pub mod gate;

use noir_field::FieldElement;
pub use gate::Gate;

#[derive(Clone)]
pub struct Circuit(pub Vec<Gate>);

// (selector_id, selector as an i128 , We don't have big int yet)
#[derive(Clone, Debug)]
pub struct Selector(pub String, pub FieldElement); //XXX(med) I guess we know it's going to be a FieldElement, so we should probably find a way to give it FieldElement directly instead of Polynomial

impl Default for Selector {
    fn default() -> Selector {
        Selector("zero".to_string(),FieldElement::zero())
    }
}
