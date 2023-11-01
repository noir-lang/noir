use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;

// Reexport bug (3384)
// use dep::private_kernel_lib::abis::membership_witness::VKMembershipWitness;
use dep::aztec::constants_gen::ROLLUP_VK_TREE_HEIGHT;
struct VKMembershipWitness{
    leaf_index : Field,
    sibling_path : [Field; ROLLUP_VK_TREE_HEIGHT]
}

// Reexport bug (3384)
// use dep::private_kernel_lib::mocked::{Proof, VerificationKey};
struct Proof;
struct VerificationKey;

struct PreviousRollupData{
    base_or_merge_rollup_public_inputs : BaseOrMergeRollupPublicInputs,
    proof : Proof,
    vk : VerificationKey,
    vk_index : u32,
    vk_sibling_path : VKMembershipWitness,
}
