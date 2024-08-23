use crate::abis::previous_rollup_block_data::PreviousRollupBlockData;
use dep::types::constants::BLOCK_ROOT_ROLLUP_INDEX;
use dep::types::tests::fixtures;
use dep::types::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use dep::types::merkle_tree::MembershipWitness;

pub fn default_previous_rollup_block_data() -> [PreviousRollupBlockData; 2] {
    let mut previous_rollup_data = [PreviousRollupBlockData::empty(); 2];

    let vk_index = BLOCK_ROOT_ROLLUP_INDEX;
    let vk_tree = fixtures::vk_tree::get_vk_merkle_tree();
    let vk_hash = vk_tree.leaves[vk_index];
    let vk_path = vk_tree.get_sibling_path(vk_index);
    let vk_tree_root = vk_tree.get_root();

    previous_rollup_data[0].block_root_or_block_merge_public_inputs.vk_tree_root = vk_tree_root;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.vk_tree_root = vk_tree_root;

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

    previous_rollup_data[0].block_root_or_block_merge_public_inputs.previous_archive = AppendOnlyTreeSnapshot {
        root: 0,
        next_available_leaf_index: 0
    };
    previous_rollup_data[0].block_root_or_block_merge_public_inputs.new_archive = AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.previous_archive= AppendOnlyTreeSnapshot {
        root: 1,
        next_available_leaf_index: 1
    };
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.new_archive = AppendOnlyTreeSnapshot {
        root: 2,
        next_available_leaf_index: 2
    };

    previous_rollup_data[0].block_root_or_block_merge_public_inputs.previous_block_hash = 1;
    previous_rollup_data[0].block_root_or_block_merge_public_inputs.end_block_hash = 2;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.previous_block_hash = 2;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.end_block_hash = 3;

    // previous_rollup_data is from one block_root circuit => ecompasses a single block (block 1)
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.start_global_variables.block_number = 1;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.end_global_variables.block_number = 1;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.start_global_variables.timestamp = 2;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.end_global_variables.timestamp = 2;

    previous_rollup_data[0].block_root_or_block_merge_public_inputs.out_hash = 1;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.out_hash = 2;

    previous_rollup_data[0].block_root_or_block_merge_public_inputs.fees[0].value = 10;
    previous_rollup_data[1].block_root_or_block_merge_public_inputs.fees[0].value = 15;

    previous_rollup_data
}
