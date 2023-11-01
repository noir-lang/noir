use crate::abis::previous_rollup_data::PreviousRollupData;
use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;

struct MergeRollupInputs{
    // TODO(Kev): Why is this 2?
    previous_rollup_data : [PreviousRollupData; 2]
}

impl MergeRollupInputs {
    pub fn merge_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        let zeroed = dep::std::unsafe::zeroed();
        zeroed
    }
}