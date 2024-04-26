use dep::types::{traits::Empty, recursion::{verification_key::VerificationKey, proof::RecursiveProof}};
use crate::parity_public_inputs::ParityPublicInputs;

struct RootParityInput {
    proof: RecursiveProof,
    verification_key: VerificationKey,
    public_inputs: ParityPublicInputs,
}

impl Empty for RootParityInput {
    fn empty() -> Self {
        RootParityInput {
            proof: RecursiveProof::empty(),
            verification_key: VerificationKey::empty(),
            public_inputs: ParityPublicInputs::empty(),
        }
    }
}
