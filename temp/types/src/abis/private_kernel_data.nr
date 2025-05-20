use crate::{
    abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    constants::{PRIVATE_KERNEL_RESET_INDEX, PROOF_TYPE_PG, VK_TREE_HEIGHT},
    merkle_tree::membership::assert_check_membership,
    proof::{traits::Verifiable, verification_key::ClientIVCVerificationKey},
    utils::arrays::find_index_hint,
};

pub struct PrivateKernelData {
    pub public_inputs: PrivateKernelCircuitPublicInputs,
    pub vk: ClientIVCVerificationKey,
    pub vk_index: u32,
    pub vk_path: [Field; VK_TREE_HEIGHT],
}

impl PrivateKernelData {
    pub fn validate_in_vk_tree<let N: u32>(self, allowed_indices: [u32; N]) {
        self.vk.check_hash();

        let index_in_allowed_list = if self.vk_index >= PRIVATE_KERNEL_RESET_INDEX {
            // Kernel circuits only need to include PRIVATE_KERNEL_RESET_INDEX in the list to allow all private kernel reset variants.
            PRIVATE_KERNEL_RESET_INDEX
        } else {
            self.vk_index
        };
        // Safety: find_index_hint should return an index into allowed_indices where `index == index_in_allowed_list`.
        // The assertion below then verifies that the condition is met.
        let index_hint = unsafe {
            find_index_hint(allowed_indices, |index: u32| index == index_in_allowed_list)
        };
        assert(index_hint < N, "Invalid vk index");
        assert_eq(allowed_indices[index_hint], index_in_allowed_list, "Invalid vk index");

        assert_check_membership(
            self.vk.hash,
            self.vk_index as Field,
            self.vk_path,
            self.public_inputs.constants.vk_tree_root,
        );
    }
}

impl Verifiable for PrivateKernelData {
    fn verify(self) {
        // Client IVC public inputs are linked in the backend via the databus
        std::verify_proof_with_type(self.vk.key, [], [], 0, PROOF_TYPE_PG);
    }
}

pub struct PrivateKernelDataWithoutPublicInputs {
    vk: ClientIVCVerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl PrivateKernelDataWithoutPublicInputs {
    pub fn to_private_kernel_data(
        self,
        public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> PrivateKernelData {
        PrivateKernelData {
            public_inputs,
            vk: self.vk,
            vk_index: self.vk_index,
            vk_path: self.vk_path,
        }
    }
}
