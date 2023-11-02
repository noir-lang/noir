// TODO(Kev): This constant is also defined in private-kernel-lib
use dep::aztec::constants_gen::NUM_FIELDS_PER_SHA256;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::abis::constant_rollup_data::ConstantRollupData;

// Re-export bug 3384
// use dep::private_kernel_lib::mocked::AggregationObject;
struct AggregationObject;

global BASE_ROLLUP_TYPE = 0;
global MERGE_ROLLUP_TYPE = 1;

struct BaseOrMergeRollupPublicInputs {
    // rollup_type is either 0 (base) or 1 (merge)
    // TODO(Kev): Why is this a u32 instead of a u8/u16?
    rollup_type : u32,
    // subtree height is always 0 for base.
    // so that we always pass-in two base/merge circuits of the same height into the next level of recursion
    rollup_subtree_height : Field,
    end_aggregation_object : AggregationObject,
    constants : ConstantRollupData,

    start_note_hash_tree_snapshot : AppendOnlyTreeSnapshot,
    end_note_hash_tree_snapshot : AppendOnlyTreeSnapshot,
    
    start_nullifier_tree_snapshot : AppendOnlyTreeSnapshot,
    end_nullifier_tree_snapshot : AppendOnlyTreeSnapshot,

    start_contract_tree_snapshot : AppendOnlyTreeSnapshot,
    end_contract_tree_snapshot : AppendOnlyTreeSnapshot,

    start_public_data_tree_root : Field,
    end_public_data_tree_root : Field,
    
    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    // TODO(Kev): Want to make this a U128, but Re-export bug 3384

    // We hash public inputs to make them constant-sized (to then be unpacked on-chain)
    calldata_hash : [Field; 2],
}
