use dep::types::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    partial_state_reference::PartialStateReference,
    mocked::AggregationObject,
    traits::Empty
};
use crate::abis::constant_rollup_data::ConstantRollupData;

global BASE_ROLLUP_TYPE = 0;
global MERGE_ROLLUP_TYPE = 1;

struct BaseOrMergeRollupPublicInputs {
    // rollup_type is either 0 (base) or 1 (merge)
    // TODO(Kev): Why is this a u32 instead of a u8/u16?
    rollup_type : u32,
    // subtree height is always 0 for base.
    // so that we always pass-in two base/merge circuits of the same height into the next level of recursion
    height_in_block_tree : Field,
    aggregation_object : AggregationObject,
    constants : ConstantRollupData,

    start: PartialStateReference,
    end: PartialStateReference,

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    // U128 isn't safe if it's an input to the circuit (it won't automatically constrain the witness)
    // So we want to constrain it when casting these fields to U128

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    txs_effects_hash : Field,
    out_hash : Field, 
}

impl Empty for BaseOrMergeRollupPublicInputs {
    fn empty() -> Self {
        BaseOrMergeRollupPublicInputs {
            rollup_type : 0 as u32,
            height_in_block_tree : 0,
            aggregation_object : AggregationObject::empty(),
            constants : ConstantRollupData::empty(),
            start: PartialStateReference::empty(),
            end: PartialStateReference::empty(),
            txs_effects_hash : 0,
            out_hash : 0, 
        }
    }
}