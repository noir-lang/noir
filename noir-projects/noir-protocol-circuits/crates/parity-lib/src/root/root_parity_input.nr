use dep::types::{
    traits::Empty,
    recursion::{verification_key::VerificationKey, proof::RecursiveProof, traits::Verifiable}
};
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

impl Verifiable for RootParityInput {
  fn verify(self) {
    let inputs = ParityPublicInputs::serialize(self.public_inputs);
    std::verify_proof(
        self.verification_key.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.verification_key.hash
    );
  }
}
