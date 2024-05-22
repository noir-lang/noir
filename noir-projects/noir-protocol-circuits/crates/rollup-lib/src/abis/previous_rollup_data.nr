use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::types::{
    constants::ROLLUP_VK_TREE_HEIGHT,
    recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable},
    traits::Empty, merkle_tree::MembershipWitness
};

struct PreviousRollupData{
    base_or_merge_rollup_public_inputs : BaseOrMergeRollupPublicInputs,
    proof : NestedRecursiveProof,
    vk : VerificationKey,
    vk_index : u32,
    vk_sibling_path : MembershipWitness<ROLLUP_VK_TREE_HEIGHT>,
}

impl Verifiable for PreviousRollupData {
  fn verify(self) {
    let inputs = BaseOrMergeRollupPublicInputs::serialize(self.base_or_merge_rollup_public_inputs);
    dep::std::verify_proof(
        self.vk.key.as_slice(),
        self.proof.fields.as_slice(),
        inputs.as_slice(),
        self.vk.hash
    );
  }
}

impl Empty for PreviousRollupData {
    fn empty() -> Self {
        PreviousRollupData {
            base_or_merge_rollup_public_inputs: BaseOrMergeRollupPublicInputs::empty(),
            proof : NestedRecursiveProof::empty(),
            vk : VerificationKey::empty(),
            vk_index : 0 as u32,
            vk_sibling_path : MembershipWitness::empty(),
        }
    }
}
