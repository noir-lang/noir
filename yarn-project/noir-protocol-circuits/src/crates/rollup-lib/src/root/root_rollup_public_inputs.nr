use crate::abis::nullifier_leaf_preimage::NullifierLeafPreimage;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::abis::constant_rollup_data::ConstantRollupData;
use crate::abis::global_variables::GlobalVariables;
use dep::types::constants::{
    NUM_FIELDS_PER_SHA256
};
use dep::types::mocked::AggregationObject;


struct RootRollupPublicInputs {
    // All below are shared between the base and merge rollups
    end_aggregation_object : AggregationObject,

    global_variables : GlobalVariables,

    start_note_hash_tree_snapshot : AppendOnlyTreeSnapshot,
    end_note_hash_tree_snapshot : AppendOnlyTreeSnapshot,

    start_nullifier_tree_snapshot : AppendOnlyTreeSnapshot,
    end_nullifier_tree_snapshot : AppendOnlyTreeSnapshot,

    start_contract_tree_snapshot : AppendOnlyTreeSnapshot,
    end_contract_tree_snapshot : AppendOnlyTreeSnapshot,

    start_public_data_tree_snapshot : AppendOnlyTreeSnapshot,
    end_public_data_tree_snapshot : AppendOnlyTreeSnapshot,

    start_l1_to_l2_messages_tree_snapshot : AppendOnlyTreeSnapshot,
    end_l1_to_l2_messages_tree_snapshot : AppendOnlyTreeSnapshot,

    start_archive_snapshot : AppendOnlyTreeSnapshot,
    end_archive_snapshot : AppendOnlyTreeSnapshot,

    calldata_hash : [Field; NUM_FIELDS_PER_SHA256],
    l1_to_l2_messages_hash : [Field; NUM_FIELDS_PER_SHA256],
}