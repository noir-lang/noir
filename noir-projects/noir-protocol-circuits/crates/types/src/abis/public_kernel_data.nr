use crate::mocked::{Proof, VerificationKey};
use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, KernelCircuitPublicInputs};

struct PublicKernelData {
    public_inputs: PublicKernelCircuitPublicInputs,
    proof: Proof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}
