use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::{PublicKernelCircuitPublicInputs, KernelCircuitPublicInputs};
use crate::recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable};

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
