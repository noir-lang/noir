use dep::types::{
    mocked::Proof,
    traits::Empty
};
use crate::parity_public_inputs::ParityPublicInputs;

struct RootParityInput {
    proof: Proof,
    public_inputs: ParityPublicInputs,
}

impl Empty for RootParityInput {
    fn empty() -> Self {
        RootParityInput {
            proof: Proof::empty(),
            public_inputs: ParityPublicInputs::empty(),
        }
    }
}