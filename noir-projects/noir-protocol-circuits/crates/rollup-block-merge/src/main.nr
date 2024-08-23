use dep::rollup_lib::block_merge::{BlockMergeRollupInputs, BlockRootOrBlockMergePublicInputs};

fn main(inputs: BlockMergeRollupInputs) -> pub BlockRootOrBlockMergePublicInputs {
    inputs.block_merge_rollup_circuit()
}
