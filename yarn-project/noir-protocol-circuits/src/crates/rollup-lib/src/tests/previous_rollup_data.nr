use crate::abis::base_or_merge_rollup_public_inputs::BASE_ROLLUP_TYPE;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::abis::previous_rollup_data::PreviousRollupData;

pub fn default_previous_rollup_data() -> [PreviousRollupData; 2] {
    let mut previous_rollup_data: [PreviousRollupData; 2] = dep::std::unsafe::zeroed();

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_contract_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end_contract_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start_contract_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_contract_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_public_data_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 1
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end_public_data_tree_snapshot =AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 2
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start_public_data_tree_snapshot = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 2
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_public_data_tree_snapshot =AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 3
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = BASE_ROLLUP_TYPE;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = BASE_ROLLUP_TYPE;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_subtree_height = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_subtree_height = 1;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash = [0, 1];
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash = [2, 3];

    previous_rollup_data
}
