use crate::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    constants::STATE_REFERENCE_LENGTH,
    partial_state_reference::PartialStateReference,
    traits::{Deserialize, Empty, Serialize},
};
use super::utils::arrays::subarray;

pub struct StateReference {
    pub l1_to_l2_message_tree: AppendOnlyTreeSnapshot,
    pub partial: PartialStateReference,
}

impl Eq for StateReference {
    fn eq(self, other: StateReference) -> bool {
        self.l1_to_l2_message_tree.eq(other.l1_to_l2_message_tree) & self.partial.eq(other.partial)
    }
}

impl Serialize<STATE_REFERENCE_LENGTH> for StateReference {
    fn serialize(self) -> [Field; STATE_REFERENCE_LENGTH] {
        let mut fields: BoundedVec<Field, STATE_REFERENCE_LENGTH> = BoundedVec::new();

        fields.extend_from_array(self.l1_to_l2_message_tree.serialize());
        fields.extend_from_array(self.partial.serialize());

        fields.storage()
    }
}

impl Deserialize<STATE_REFERENCE_LENGTH> for StateReference {
    fn deserialize(serialized: [Field; STATE_REFERENCE_LENGTH]) -> StateReference {
        let mut offset = 0;

        let l1_to_l2_message_tree_fields = subarray(serialized, offset);
        offset = offset + l1_to_l2_message_tree_fields.len();

        let partial_fields = subarray(serialized, offset);

        StateReference {
            l1_to_l2_message_tree: AppendOnlyTreeSnapshot::deserialize(
                l1_to_l2_message_tree_fields,
            ),
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

#[test]
fn serialization_of_empty() {
    let state = StateReference::empty();
    let _serialized = state.serialize();
    let _deserialized = StateReference::deserialize(_serialized);
}
