use dep::types::{
    traits::Empty,
    recursion::{verification_key::VerificationKey, proof::NestedRecursiveProof, traits::Verifiable}
};
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

impl Verifiable for RootRollupParityInput {
  fn verify(self) {
    let inputs = ParityPublicInputs::serialize(self.public_inputs);
    dep::std::verify_proof(
        self.verification_key.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.verification_key.hash
    );
  }
}
