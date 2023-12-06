use crate::mocked::{Proof, VerificationKey};
use crate::constants::VK_TREE_HEIGHT;
use crate::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputs;

//TODO(sean): left a note saying that this should not be called Previous
struct PreviousKernelData {
    // TODO(David): Left a note asking if we need this due to it
    // already being in the proof.
    public_inputs : KernelCircuitPublicInputs,
    // TODO(David): Mentions the dichotomy between a proof created for the 
    // circuit, which is a sequence of field elements, versus a proof
    // created for solidity/native verification which is a collection of bytes.
    // Kev: I've been questioning if we _need_ the sequence of field elements.
    // It makes verification cheaper, though I have not tested how much cheaper.
    // Removing it would also reduce complexity on the Noir side, as we have 
    // special methods to convert "inner proofs" into sequence of field elements. 
    proof : Proof,

    vk : VerificationKey,

    // TODO(Mike): left a note saying : this index and path are meant to be those of a leaf within the tree of _kernel circuit_ vks; not the tree
    // of functions within the contract tree.
    vk_index : u32,
    vk_path : [Field; VK_TREE_HEIGHT],


}
