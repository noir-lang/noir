use crate::root::root_rollup_inputs::RootRollupInputs;
use crate::tests::previous_rollup_block_data::default_previous_rollup_block_data;

pub fn default_root_rollup_inputs() -> RootRollupInputs {
    let mut inputs = RootRollupInputs::empty();

    inputs.previous_rollup_data = default_previous_rollup_block_data();

    inputs
}
