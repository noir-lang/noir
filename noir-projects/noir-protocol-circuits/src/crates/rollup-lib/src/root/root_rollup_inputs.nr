use crate::abis::previous_rollup_data::PreviousRollupData;
use crate::abis::constant_rollup_data::ConstantRollupData;
use dep::types::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH, ARCHIVE_HEIGHT}
};

struct RootRollupInputs {
    // All below are shared between the base and merge rollups
    previous_rollup_data : [PreviousRollupData; 2],

    // inputs required to process l1 to l2 messages
    new_l1_to_l2_messages : [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
    new_l1_to_l2_message_tree_root_sibling_path : [Field; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],

    start_l1_to_l2_message_tree_snapshot : AppendOnlyTreeSnapshot,

    // inputs required to add the block hash
    start_archive_snapshot : AppendOnlyTreeSnapshot,
    new_archive_sibling_path : [Field; ARCHIVE_HEIGHT],
}
