use crate::abis::base_or_merge_rollup_public_inputs::BASE_ROLLUP_TYPE;
use crate::abis::previous_rollup_data::PreviousRollupData;
use dep::types::constants::BASE_ROLLUP_INDEX;
use dep::types::tests::fixtures;
use dep::types::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use dep::types::merkle_tree::MembershipWitness;

pub fn default_previous_rollup_data() -> [PreviousRollupData; 2] {
    let mut previous_rollup_data = [PreviousRollupData::empty(); 2];

    let vk_index = BASE_ROLLUP_INDEX;
    let vk_tree = fixtures::vk_tree::get_vk_merkle_tree();
    let vk_hash = vk_tree.leaves[vk_index];
    let vk_path = vk_tree.get_sibling_path(vk_index);
    let vk_tree_root = vk_tree.get_root();

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.vk_tree_root = vk_tree_root;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.vk_tree_root = vk_tree_root;

    previous_rollup_data[0].vk.hash = vk_hash;
    previous_rollup_data[1].vk.hash = vk_hash;

    previous_rollup_data[0].vk_witness = MembershipWitness {
        leaf_index: vk_index as Field,
        sibling_path: vk_path
    };
    previous_rollup_data[1].vk_witness =  MembershipWitness {
        leaf_index: vk_index as Field,
        sibling_path: vk_path
    };

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

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.num_txs = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.num_txs = 1;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.txs_effects_hash = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.txs_effects_hash = 2;

    previous_rollup_data[0].base_or_merge_rollup_public_inputs.out_hash = 1;
    previous_rollup_data[1].base_or_merge_rollup_public_inputs.out_hash = 2;

    previous_rollup_data
}
