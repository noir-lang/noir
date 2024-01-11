use dep::types::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    constants::NUM_FIELDS_PER_SHA256,
    partial_state_reference::PartialStateReference,
};
use crate::abis::constant_rollup_data::ConstantRollupData;
use dep::types::mocked::AggregationObject;

global BASE_ROLLUP_TYPE = 0;
global MERGE_ROLLUP_TYPE = 1;

struct BaseOrMergeRollupPublicInputs {
    // rollup_type is either 0 (base) or 1 (merge)
    // TODO(Kev): Why is this a u32 instead of a u8/u16?
    rollup_type : u32,
    // subtree height is always 0 for base.
    // so that we always pass-in two base/merge circuits of the same height into the next level of recursion
    // TODO(benesjan): rename to height_in_block_tree
    rollup_subtree_height : Field,
    aggregation_object : AggregationObject,
    constants : ConstantRollupData,

    start: PartialStateReference,
    end: PartialStateReference,

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    // U128 isn't safe if it's an input to the circuit (it won't automatically constrain the witness)
    // So we want to constrain it when casting these fields to U128

    // TODO(#3938): split this to txs_hash and out_hash
    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    calldata_hash : [Field; 2],
}
