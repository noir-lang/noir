use crate::abis::base_or_merge_rollup_public_inputs::{
    BaseOrMergeRollupPublicInputs, MERGE_ROLLUP_TYPE,
};
use dep::types::hash::accumulate_sha256;

pub fn merge_rollups(
    prev_rollup: BaseOrMergeRollupPublicInputs,
    next_rollup: BaseOrMergeRollupPublicInputs,
) -> BaseOrMergeRollupPublicInputs {
    let num_txs = prev_rollup.num_txs + next_rollup.num_txs;

    let out_hash = accumulate_sha256([prev_rollup.out_hash, next_rollup.out_hash]);

    let accumulated_fees = prev_rollup.accumulated_fees + next_rollup.accumulated_fees;

    let accumulated_mana_used =
        prev_rollup.accumulated_mana_used + next_rollup.accumulated_mana_used;

    BaseOrMergeRollupPublicInputs {
        rollup_type: MERGE_ROLLUP_TYPE,
        num_txs,
        constants: prev_rollup.constants,
        start: prev_rollup.start,
        end: next_rollup.end,
        start_sponge_blob: prev_rollup.start_sponge_blob,
        end_sponge_blob: next_rollup.end_sponge_blob,
        out_hash,
        accumulated_fees,
        accumulated_mana_used,
    }
}
