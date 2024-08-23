use crate::block_merge::block_merge_rollup_inputs::BlockMergeRollupInputs;
use crate::tests::previous_rollup_block_data::default_previous_rollup_block_data;

pub fn default_block_merge_rollup_inputs() -> BlockMergeRollupInputs {
    let mut inputs = BlockMergeRollupInputs::empty();

    inputs.previous_rollup_data = default_previous_rollup_block_data();

    inputs
}
