use crate::abis::base_or_merge_rollup_public_inputs::BASE_ROLLUP_TYPE;
use crate::abis::previous_rollup_data::PreviousRollupData;
use dep::types::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;

pub fn default_previous_rollup_data() -> [PreviousRollupData; 2] {
    let mut previous_rollup_data: [PreviousRollupData; 2] = dep::std::unsafe::zeroed();

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start.note_hash_tree = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.note_hash_tree = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.note_hash_tree = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.note_hash_tree = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start.nullifier_tree = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.nullifier_tree = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.nullifier_tree = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.nullifier_tree = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.start.public_data_tree = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 1
    };
    previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.public_data_tree =AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 2
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.public_data_tree = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 2
    };
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.public_data_tree = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 3
    };

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = BASE_ROLLUP_TYPE;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = BASE_ROLLUP_TYPE;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.height_in_block_tree = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.height_in_block_tree = 1;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.txs_effects_hash = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.txs_effects_hash = 2;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.out_hash = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.out_hash = 2;

    previous_rollup_data
}
