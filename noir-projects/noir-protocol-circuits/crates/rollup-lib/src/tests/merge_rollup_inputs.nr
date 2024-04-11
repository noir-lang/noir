use crate::merge::merge_rollup_inputs::MergeRollupInputs;
use crate::abis::base_or_merge_rollup_public_inputs::BASE_ROLLUP_TYPE;
use dep::types::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::tests::previous_rollup_data::default_previous_rollup_data;

pub fn default_merge_rollup_inputs() -> MergeRollupInputs {
    let mut inputs = MergeRollupInputs::empty();

    inputs.previous_rollup_data = default_previous_rollup_data();

    inputs
}
