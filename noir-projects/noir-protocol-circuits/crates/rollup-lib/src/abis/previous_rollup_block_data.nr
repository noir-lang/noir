use crate::abis::block_root_or_block_merge_public_inputs::BlockRootOrBlockMergePublicInputs;
use dep::types::{
    constants::VK_TREE_HEIGHT,
    recursion::{proof::NestedRecursiveProof, verification_key::VerificationKey, traits::Verifiable},
    traits::Empty, merkle_tree::MembershipWitness, merkle_tree::membership::assert_check_membership,
    utils::arrays::find_index_hint
};

struct PreviousRollupBlockData {
    block_root_or_block_merge_public_inputs: BlockRootOrBlockMergePublicInputs,
    proof: NestedRecursiveProof,
    vk: VerificationKey,
    vk_witness: MembershipWitness<VK_TREE_HEIGHT>,
}

impl Verifiable for PreviousRollupBlockData {
  fn verify(self) {
    let inputs = BlockRootOrBlockMergePublicInputs::serialize(self.block_root_or_block_merge_public_inputs);
    std::verify_proof(
        self.vk.key,
        self.proof.fields,
        inputs,
        self.vk.hash
    );
  }
}

impl Empty for PreviousRollupBlockData {
    fn empty() -> Self {
        PreviousRollupBlockData {
            block_root_or_block_merge_public_inputs: BlockRootOrBlockMergePublicInputs::empty(),
            proof: NestedRecursiveProof::empty(),
            vk: VerificationKey::empty(),
            vk_witness: MembershipWitness::empty(),
        }
    }
}

impl PreviousRollupBlockData {
    fn validate_in_vk_tree<let N: u32>(self, allowed_indices: [u32; N]) {
        let leaf_index = self.vk_witness.leaf_index as u32;
        let index_hint = find_index_hint(allowed_indices, |index: u32| index == leaf_index);
        assert_eq(allowed_indices[index_hint], leaf_index, "Invalid vk index");

        assert_check_membership(
            self.vk.hash,
            self.vk_witness.leaf_index,
            self.vk_witness.sibling_path,
            self.block_root_or_block_merge_public_inputs.vk_tree_root
        );
    }
}
