use crate::{
    abis::append_only_tree_snapshot::{
        AppendOnlyTreeSnapshot,
        APPEND_ONLY_TREE_SNAPSHOT_LENGTH,
    },
    partial_state_reference::{
        PartialStateReference,
        PARTIAL_STATE_REFERENCE_LENGTH,
    },
    traits::Empty,
    utils::{
        arr_copy_slice,
    },
};

struct StateReference {
    l1_to_l2_message_tree: AppendOnlyTreeSnapshot,
    partial: PartialStateReference,
}

global STATE_REFERENCE_LENGTH: Field = 10; // 2 for snap + 8 for partial

impl StateReference {
    fn eq(self, other: StateReference) -> bool {
        self.l1_to_l2_message_tree.eq(other.l1_to_l2_message_tree) &
        self.partial.eq(other.partial)
    }

    fn serialize(self) -> [Field; STATE_REFERENCE_LENGTH] {
        let mut fields: BoundedVec<Field, STATE_REFERENCE_LENGTH> = BoundedVec::new(0);

        fields.extend_from_array(self.l1_to_l2_message_tree.serialize());
        fields.extend_from_array(self.partial.serialize());

        fields.storage
    }

    fn deserialize(serialized: [Field; STATE_REFERENCE_LENGTH]) -> StateReference {
        let mut offset = 0;

        let l1_to_l2_message_tree_fields = arr_copy_slice(serialized, [0; APPEND_ONLY_TREE_SNAPSHOT_LENGTH], offset);
        offset = offset + APPEND_ONLY_TREE_SNAPSHOT_LENGTH;

        let partial_fields = arr_copy_slice(serialized, [0; PARTIAL_STATE_REFERENCE_LENGTH], offset);

        StateReference {
            l1_to_l2_message_tree: AppendOnlyTreeSnapshot::deserialize(l1_to_l2_message_tree_fields),
            partial: PartialStateReference::deserialize(partial_fields),
        }
    }
}

impl Empty for StateReference {
    fn empty() -> Self {
        Self {
            l1_to_l2_message_tree: AppendOnlyTreeSnapshot::zero(),
            partial: PartialStateReference::empty(),
        }
    }
}
