use crate::lib::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use crate::lib::abis::previous_rollup_data::PreviousRollupData;
use crate::lib::merge::utils::{
    merge_rollups::merge_rollups, validate_consecutive_rollups::validate_consecutive_rollups,
};
use crate::types::{
    constants::{MERGE_ROLLUP_INDEX, PRIVATE_BASE_ROLLUP_VK_INDEX, PUBLIC_BASE_ROLLUP_VK_INDEX},
    proof::traits::Verifiable,
    traits::Empty,
};

global ALLOWED_PREVIOUS_CIRCUITS: [u32; 3] =
    [MERGE_ROLLUP_INDEX, PRIVATE_BASE_ROLLUP_VK_INDEX, PUBLIC_BASE_ROLLUP_VK_INDEX];

pub(crate)struct MergeRollupInputs {
    pub(crate)previous_rollup_data: [PreviousRollupData; 2],
}

impl Empty for MergeRollupInputs {
    fn empty() -> Self {
        MergeRollupInputs { previous_rollup_data: [PreviousRollupData::empty(); 2] }
    }
}

impl MergeRollupInputs {
    pub(crate)fn merge_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        // Verify the previous rollup proofs
        if !dep::std::runtime::is_unconstrained() {
            self.previous_rollup_data[0].verify();
            self.previous_rollup_data[0].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);

            self.previous_rollup_data[1].verify();
            self.previous_rollup_data[1].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        let left = self.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
        let right = self.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

        validate_consecutive_rollups(left, right);

        merge_rollups(left, right)
    }
}
