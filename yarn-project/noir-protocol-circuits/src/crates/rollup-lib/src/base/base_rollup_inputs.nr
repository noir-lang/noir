use crate::abis::nullifier_leaf_preimage::NullifierLeafPreimage;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::abis::constant_rollup_data::ConstantRollupData;
use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::aztec::constants_gen::{
    MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP,
    NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
    NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
    CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP,
    MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP,
    PUBLIC_DATA_TREE_HEIGHT,
    KERNELS_PER_BASE_ROLLUP,
};
use dep::types::abis::previous_kernel_data::PreviousKernelData;
use dep::types::abis::membership_witness::NullifierMembershipWitness;
use dep::types::abis::membership_witness::HistoricBlocksTreeRootMembershipWitness;

struct BaseRollupInputs {
    kernel_data: [PreviousKernelData; KERNELS_PER_BASE_ROLLUP],
    start_note_hash_tree_snapshot: AppendOnlyTreeSnapshot,
    start_nullifier_tree_snapshot: AppendOnlyTreeSnapshot,
    start_contract_tree_snapshot: AppendOnlyTreeSnapshot,
    start_public_data_tree_root: Field,
    start_historic_blocks_tree_snapshot: AppendOnlyTreeSnapshot,

    low_nullifier_leaf_preimages: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
    low_nullifier_membership_witness: [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the above snapshots' `next_available_leaf_index` values.
    new_commitments_subtree_sibling_path: [Field; NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH],
    new_nullifiers_subtree_sibling_path: [Field; NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH],
    new_contracts_subtree_sibling_path: [Field; CONTRACT_SUBTREE_SIBLING_PATH_LENGTH],
    new_public_data_update_requests_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP],
    new_public_data_reads_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP],
    
    historic_blocks_tree_root_membership_witnesses: [HistoricBlocksTreeRootMembershipWitness; KERNELS_PER_BASE_ROLLUP],
    
    constants: ConstantRollupData,
}

impl BaseRollupInputs {
    pub fn base_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        let zeroed = dep::std::unsafe::zeroed();
        zeroed
    }
}