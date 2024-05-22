use dep::rollup_lib::base::{BaseRollupInputs, BaseOrMergeRollupPublicInputs};

#[recursive]
fn main(inputs: BaseRollupInputs) -> pub BaseOrMergeRollupPublicInputs {
    inputs.base_rollup_circuit()
}
