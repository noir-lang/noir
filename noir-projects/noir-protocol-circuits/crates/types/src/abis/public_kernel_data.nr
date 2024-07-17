use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, KernelCircuitPublicInputs};
use crate::recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable};
use crate::merkle_tree::membership::assert_check_membership;
use crate::utils::arrays::find_index_hint;

struct PublicKernelData {
    public_inputs: PublicKernelCircuitPublicInputs,
    proof: NestedRecursiveProof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl Verifiable for PublicKernelData {
  fn verify(self) {
    let inputs = PublicKernelCircuitPublicInputs::serialize(self.public_inputs);
    std::verify_proof(
        self.vk.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.vk.hash
    );
  }
}

impl PublicKernelData {
    fn validate_in_vk_tree<let N: u32>(self, allowed_indices: [u32; N]) {
        let index_hint = find_index_hint(allowed_indices, self.vk_index);
        assert_eq(allowed_indices[index_hint], self.vk_index, "Invalid vk index");

        assert_check_membership(
            self.vk.hash,
            self.vk_index as Field,
            self.vk_path,
            self.public_inputs.constants.vk_tree_root
        );
    }
}

