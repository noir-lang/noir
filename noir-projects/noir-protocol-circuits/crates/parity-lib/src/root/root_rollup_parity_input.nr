use dep::types::{traits::Empty, recursion::{verification_key::VerificationKey, proof::NestedRecursiveProof}};
use crate::parity_public_inputs::ParityPublicInputs;

struct RootRollupParityInput {
    proof: NestedRecursiveProof,
    verification_key: VerificationKey,
    public_inputs: ParityPublicInputs,
}

impl Empty for RootRollupParityInput {
    fn empty() -> Self {
        RootRollupParityInput {
            proof: NestedRecursiveProof::empty(),
            verification_key: VerificationKey::empty(),
            public_inputs: ParityPublicInputs::empty(),
        }
    }
}
