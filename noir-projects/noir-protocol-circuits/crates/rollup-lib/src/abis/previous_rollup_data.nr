use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::types::{
    constants::ROLLUP_VK_TREE_HEIGHT, mocked::{Proof, VerificationKey}, traits::Empty,
    merkle_tree::MembershipWitness
};

struct PreviousRollupData{
    base_or_merge_rollup_public_inputs : BaseOrMergeRollupPublicInputs,
    proof : Proof,
    vk : VerificationKey,
    vk_index : u32,
    vk_sibling_path : MembershipWitness<ROLLUP_VK_TREE_HEIGHT>,
}

impl Empty for PreviousRollupData {
    fn empty() -> Self {
        PreviousRollupData {
            base_or_merge_rollup_public_inputs: BaseOrMergeRollupPublicInputs::empty(),
            proof : Proof::empty(),
            vk : VerificationKey::empty(),
            vk_index : 0 as u32,
            vk_sibling_path : MembershipWitness::empty(),
        }
    }
}
