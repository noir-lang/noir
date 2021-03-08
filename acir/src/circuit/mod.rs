pub mod gate;

pub use gate::Gate;
use noir_field::FieldElement;

use crate::native_types::Witness;

#[derive(Clone, Debug)]
pub struct Circuit {
    pub current_witness_index: u32,
    pub gates: Vec<Gate>,
    pub public_inputs: PublicInputs,
}

#[derive(Clone, Debug)]
pub struct PublicInputs(pub Vec<Witness>);

impl PublicInputs {
    /// Returns the witness index of each public input
    pub fn indices(&self) -> Vec<u32> {
        self.0
            .iter()
            .map(|witness| witness.witness_index() as u32)
            .collect()
    }
}
#[derive(Clone, Debug)]
pub struct Selector(pub String, pub FieldElement);

impl Default for Selector {
    fn default() -> Selector {
        Selector("zero".to_string(), FieldElement::zero())
    }
}
