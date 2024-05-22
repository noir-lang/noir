use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputs;
use crate::recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable};

struct KernelData {
    public_inputs: KernelCircuitPublicInputs,
    proof: NestedRecursiveProof,
    vk: VerificationKey,
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl Verifiable for KernelData {
  fn verify(self) {
    let inputs = KernelCircuitPublicInputs::serialize(self.public_inputs);
    dep::std::verify_proof(
        self.vk.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.vk.hash
    );
  }
}
