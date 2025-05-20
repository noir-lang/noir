use crate::lib::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use crate::types::{
    constants::{PROOF_TYPE_ROLLUP_HONK, VK_TREE_HEIGHT},
    merkle_tree::{membership::assert_check_membership},
    proof::{
        rollup_recursive_proof::NestedRecursiveProof,
        traits::Verifiable,
        verification_key::{RollupHonkVerificationKey, VerificationKey},
    },
    traits::{Empty, Serialize},
    utils::arrays::find_index_hint,
};

pub(crate)struct PreviousRollupData {
    pub(crate) base_or_merge_rollup_public_inputs: BaseOrMergeRollupPublicInputs,
    pub(crate) proof: NestedRecursiveProof,
    pub(crate) vk: RollupHonkVerificationKey,
    pub(crate) vk_witness: crate::types::merkle_tree::MembershipWitness<VK_TREE_HEIGHT>,
}

impl Verifiable for PreviousRollupData {
    fn verify(self) {
        let inputs =
            BaseOrMergeRollupPublicInputs::serialize(self.base_or_merge_rollup_public_inputs);
        std::verify_proof_with_type(
            self.vk.key,
            self.proof.fields,
            inputs,
            self.vk.hash,
            PROOF_TYPE_ROLLUP_HONK,
        );
    }
}

impl Empty for PreviousRollupData {
    fn empty() -> Self {
        PreviousRollupData {
            base_or_merge_rollup_public_inputs: BaseOrMergeRollupPublicInputs::empty(),
            proof: NestedRecursiveProof::empty(),
            vk: VerificationKey::empty(),
            vk_witness: crate::types::merkle_tree::MembershipWitness::empty(),
        }
    }
}

impl PreviousRollupData {
    pub(crate)fn validate_in_vk_tree<let N: u32>(self, allowed_indices: [u32; N]) {
        self.vk.check_hash();

        let leaf_index = self.vk_witness.leaf_index as u32;
        // Safety: This is a hint which is constrained by the asserts below.
        let index_hint =
            unsafe { find_index_hint(allowed_indices, |index: u32| index == leaf_index) };
        assert(index_hint < N, "Invalid vk index");
        assert_eq(allowed_indices[index_hint], leaf_index, "Invalid vk index");

        assert_check_membership(
            self.vk.hash,
            self.vk_witness.leaf_index,
            self.vk_witness.sibling_path,
            self.base_or_merge_rollup_public_inputs.constants.vk_tree_root,
        );
    }
}
