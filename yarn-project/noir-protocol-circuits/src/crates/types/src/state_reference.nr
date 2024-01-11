use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::partial_state_reference::PartialStateReference;

struct StateReference {
    l1_to_l2_message_tree: AppendOnlyTreeSnapshot,
    partial: PartialStateReference,
}
