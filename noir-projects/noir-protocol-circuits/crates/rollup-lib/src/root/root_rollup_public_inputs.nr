use dep::types::{abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot, header::Header};

struct RootRollupPublicInputs {
    // Snapshot of archive tree after this block/rollup been processed
    archive: AppendOnlyTreeSnapshot,

    // New block header
    header: Header,
}
