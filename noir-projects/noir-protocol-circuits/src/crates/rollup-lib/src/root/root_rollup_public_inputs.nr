use dep::types::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, global_variables::GlobalVariables},
    constants::{NUM_FIELDS_PER_SHA256}, header::Header
};
use dep::types::mocked::AggregationObject;

struct RootRollupPublicInputs {
    // All below are shared between the base and merge rollups
    aggregation_object : AggregationObject,

    // Snapshot of archive tree after this block/rollup been processed
    archive: AppendOnlyTreeSnapshot,

    // New block header
    header: Header,

    // TODO(#3938): Nuke this once body hash/calldata hash is updated
    l1_to_l2_messages_hash : [Field; NUM_FIELDS_PER_SHA256],
}
