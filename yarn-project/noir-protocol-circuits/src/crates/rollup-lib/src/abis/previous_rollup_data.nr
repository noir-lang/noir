use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::types::abis::membership_witness::VKMembershipWitness;
use dep::aztec::constants_gen::ROLLUP_VK_TREE_HEIGHT;
use dep::types::mocked::{Proof, VerificationKey};

struct PreviousRollupData{
    base_or_merge_rollup_public_inputs : BaseOrMergeRollupPublicInputs,
    proof : Proof,
    vk : VerificationKey,
    vk_index : u32,
    vk_sibling_path : VKMembershipWitness,
}
