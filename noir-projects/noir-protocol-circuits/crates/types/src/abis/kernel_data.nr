use crate::mocked::{Proof, VerificationKey};
use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputs;

struct KernelData {
    public_inputs: KernelCircuitPublicInputs,
    proof: Proof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}
