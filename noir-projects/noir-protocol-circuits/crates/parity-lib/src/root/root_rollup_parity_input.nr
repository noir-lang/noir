use dep::types::{
    traits::Empty,
    recursion::{verification_key::VerificationKey, proof::NestedRecursiveProof, traits::Verifiable},
    constants::{ROOT_PARITY_INDEX, VK_TREE_HEIGHT}, merkle_tree::membership::assert_check_membership
};
use crate::parity_public_inputs::ParityPublicInputs;

struct RootRollupParityInput {
    proof: NestedRecursiveProof,
    verification_key: VerificationKey,
    vk_path: [Field; VK_TREE_HEIGHT],
    public_inputs: ParityPublicInputs,
}

impl Empty for RootRollupParityInput {
    fn empty() -> Self {
        RootRollupParityInput {
            proof: NestedRecursiveProof::empty(),
            verification_key: VerificationKey::empty(),
            vk_path: [0; VK_TREE_HEIGHT],
            public_inputs: ParityPublicInputs::empty(),
        }
    }
}

impl Verifiable for RootRollupParityInput {
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

impl RootRollupParityInput {
    fn validate_in_vk_tree(self) {
        assert_check_membership(
            self.verification_key.hash,
            ROOT_PARITY_INDEX as Field,
            self.vk_path,
            self.public_inputs.vk_tree_root
        );
    }
}
