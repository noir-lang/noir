mod root_rollup_inputs;
use root_rollup_inputs::RootRollupInputs;
mod root_rollup_public_inputs;
use root_rollup_public_inputs::RootRollupPublicInputs;


impl RootRollupInputs {
    pub fn root_rollup_circuit(self) -> RootRollupPublicInputs {
        let zeroed = dep::std::unsafe::zeroed();
        zeroed
    }
}