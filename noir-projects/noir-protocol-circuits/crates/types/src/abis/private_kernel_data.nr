use crate::recursion::{verification_key::VerificationKey, proof::NestedRecursiveProof, traits::Verifiable};
use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs;
use crate::merkle_tree::membership::assert_check_membership;
use crate::utils::arrays::find_index_hint;

struct PrivateKernelData {
    // TODO(David): Left a note asking if we need this due to it
    // already being in the proof.
    public_inputs: PrivateKernelCircuitPublicInputs,

    // TODO(David): Mentions the dichotomy between a proof created for the 
    // circuit, which is a sequence of field elements, versus a proof
    // created for solidity/native verification which is a collection of bytes.
    // Kev: I've been questioning if we _need_ the sequence of field elements.
    // It makes verification cheaper, though I have not tested how much cheaper.
    // Removing it would also reduce complexity on the Noir side, as we have 
    // special methods to convert "inner proofs" into sequence of field elements. 
    proof: NestedRecursiveProof,
    vk: VerificationKey,

    // This index and path are meant to be those of a leaf within the tree of protocol circuits vks; not the tree
    // of functions within the contract tree.
    vk_index: u32,
    vk_path: [Field; VK_TREE_HEIGHT],
}

impl Verifiable for PrivateKernelData {
  fn verify(self) {
    let inputs = PrivateKernelCircuitPublicInputs::serialize(self.public_inputs);
    std::verify_proof(
        self.vk.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.vk.hash
    );
  }
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
