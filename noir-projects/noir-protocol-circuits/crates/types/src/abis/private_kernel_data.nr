use crate::recursion::{verification_key::VerificationKey, proof::NestedRecursiveProof, traits::Verifiable};
use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs;
use crate::merkle_tree::membership::assert_check_membership;
use crate::utils::arrays::find_index_hint;

struct PrivateKernelData {
    public_inputs: PrivateKernelCircuitPublicInputs,

    vk: VerificationKey,

    // This index and path are meant to be those of a leaf within the tree of protocol circuits vks; not the tree
    // of functions within the contract tree.
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl PrivateKernelData {
    fn validate_in_vk_tree<N>(self, allowed_indices: [u32; N]) {
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
