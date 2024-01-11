use crate::{
    abis::{
        append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        global_variables::GlobalVariables,
    },
    constants::NUM_FIELDS_PER_SHA256,
    state_reference::StateReference,
};

struct Header {
    last_archive: AppendOnlyTreeSnapshot,
    body_hash: [Field; NUM_FIELDS_PER_SHA256],
    state: StateReference,
    global_variables: GlobalVariables,
}
