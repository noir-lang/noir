use dep::rollup_lib::base::{BaseRollupInputs,BaseOrMergeRollupPublicInputs};

//TODO add a circuit variant
unconstrained fn main(inputs : BaseRollupInputs) -> pub BaseOrMergeRollupPublicInputs {
    inputs.base_rollup_circuit()
}