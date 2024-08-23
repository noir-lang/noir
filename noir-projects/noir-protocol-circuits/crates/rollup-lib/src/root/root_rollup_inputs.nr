use crate::{
    abis::previous_rollup_block_data::PreviousRollupBlockData, components,
    root::root_rollup_public_inputs::RootRollupPublicInputs
};
use types::{traits::Empty, constants::{BLOCK_ROOT_ROLLUP_INDEX, BLOCK_MERGE_ROLLUP_INDEX}};
// TODO(#7346): Currently unused! Will be used when batch rollup circuits are integrated.
global ALLOWED_PREVIOUS_CIRCUITS = [
    BLOCK_ROOT_ROLLUP_INDEX,
    BLOCK_MERGE_ROLLUP_INDEX,
];

struct RootRollupInputs {
    previous_rollup_data: [PreviousRollupBlockData; 2],
    prover_id: Field,
}

impl Empty for RootRollupInputs {
    fn empty() -> Self {
        RootRollupInputs {
            previous_rollup_data: [PreviousRollupBlockData::empty(); 2],
            prover_id: 0,
        }
    }
}

impl RootRollupInputs {
    pub fn root_rollup_circuit(self) -> RootRollupPublicInputs {
        // Verify the previous rollup proofs
        if !dep::std::runtime::is_unconstrained() {
            self.previous_rollup_data[0].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[0].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);

            self.previous_rollup_data[1].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[1].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        let left = self.previous_rollup_data[0].block_root_or_block_merge_public_inputs;
        let right = self.previous_rollup_data[1].block_root_or_block_merge_public_inputs;

        // TODO(#7346): Constrain block rollup structure
        // For full wonkiness (no wasted proving of padding blocks for any number of blocks, but more complex hash recalculation):
        //  components::assert_blocks_filled_from_left(left, right);
        // For variable height but balanced tree (no wasted proving only for number of blocks = power of 2, simpler hash recalculation):
        //  assert(left.num_blocks == right.num_blocks)
        //  if (num_blocks == 1) { assert(vk_witness.index == BLOCK_ROOT_ROLLUP_INDEX) } else { assert(vk_witness.index == BLOCK_MERGE_ROLLUP_INDEX)}
        // ^ Where instead of num_txs, use num_blocks = (end_global_variables.block_number - start_global_variables.block_number) + 1

        components::assert_prev_block_rollups_follow_on_from_each_other(left, right);

        let out_hash = components::compute_blocks_out_hash(self.previous_rollup_data);

        let fees = components::accumulate_blocks_fees(left, right);

        RootRollupPublicInputs {
            previous_archive: left.previous_archive,
            end_archive: right.new_archive,
            previous_block_hash: left.previous_block_hash,
            end_block_hash: right.end_block_hash,
            end_timestamp: right.end_global_variables.timestamp,
            end_block_number: right.end_global_variables.block_number,
            out_hash,
            fees,
            vk_tree_root: left.vk_tree_root,
            prover_id: self.prover_id
        }
    }
}
