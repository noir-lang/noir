use dep::rollup_lib::merge::{MergeRollupInputs, BaseOrMergeRollupPublicInputs};

#[recursive]
fn main(inputs: MergeRollupInputs) -> pub BaseOrMergeRollupPublicInputs {
    inputs.merge_rollup_circuit()
}
