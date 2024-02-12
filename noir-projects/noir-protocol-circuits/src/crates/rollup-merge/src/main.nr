use dep::rollup_lib::merge::{MergeRollupInputs, BaseOrMergeRollupPublicInputs};

fn main(inputs: MergeRollupInputs) -> pub BaseOrMergeRollupPublicInputs {
    inputs.merge_rollup_circuit()
}
