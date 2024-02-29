use dep::rollup_lib::root::{RootRollupInputs, RootRollupPublicInputs};

fn main(inputs: RootRollupInputs) -> pub RootRollupPublicInputs {
    inputs.root_rollup_circuit()
}
