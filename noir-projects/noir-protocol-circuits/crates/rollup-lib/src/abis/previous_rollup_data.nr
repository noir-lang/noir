use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::types::{
    constants::VK_TREE_HEIGHT,
    recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable},
    traits::Empty, merkle_tree::MembershipWitness, merkle_tree::membership::assert_check_membership,
    utils::arrays::find_index_hint
};

struct PreviousRollupData{
    base_or_merge_rollup_public_inputs : BaseOrMergeRollupPublicInputs,
    proof : NestedRecursiveProof,
    vk : VerificationKey,
    vk_witness : MembershipWitness<VK_TREE_HEIGHT>,
}

impl Verifiable for PreviousRollupData {
  fn verify(self) {
    let inputs = BaseOrMergeRollupPublicInputs::serialize(self.base_or_merge_rollup_public_inputs);
    std::verify_proof(
        self.vk.key,
        self.proof.fields,
        inputs,
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
            vk_witness : MembershipWitness::empty(),
        }
    }
}

impl PreviousRollupData {
    fn validate_in_vk_tree<let N: u32>(self, allowed_indices: [u32; N]) {
        let leaf_index = self.vk_witness.leaf_index as u32;
        let index_hint = find_index_hint(allowed_indices, |index: u32| index == leaf_index);
        assert_eq(allowed_indices[index_hint], leaf_index, "Invalid vk index");

        assert_check_membership(
            self.vk.hash,
            self.vk_witness.leaf_index,
            self.vk_witness.sibling_path,
            self.base_or_merge_rollup_public_inputs.constants.vk_tree_root
        );
    }
}
