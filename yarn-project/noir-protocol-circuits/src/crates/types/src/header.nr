use crate::{
    abis::{
        append_only_tree_snapshot::{
            AppendOnlyTreeSnapshot,
            APPEND_ONLY_TREE_SNAPSHOT_LENGTH,
        },
        global_variables::{
            GlobalVariables,
            GLOBAL_VARIABLES_LENGTH,
        },
    },
    constants::{
        GENERATOR_INDEX__BLOCK_HASH,
        HEADER_LENGTH,
        NUM_FIELDS_PER_SHA256,
    },
    hash::pedersen_hash,
    state_reference::{
        StateReference,
        STATE_REFERENCE_LENGTH,
    },
    traits::Empty,
    utils::{
        arr_copy_slice,
        bounded_vec::BoundedVec,
    },
};

// docs:start:header
struct Header {
    last_archive: AppendOnlyTreeSnapshot,
    body_hash: [Field; NUM_FIELDS_PER_SHA256],
    state: StateReference,
    global_variables: GlobalVariables,
}
// docs:end:header

impl Header {

    pub fn serialize(self) -> [Field; HEADER_LENGTH] {
        let mut fields: BoundedVec<Field, HEADER_LENGTH> = BoundedVec::new(0);

        fields.push_array(self.last_archive.serialize());
        fields.push_array(self.body_hash);
        fields.push_array(self.state.serialize());
        fields.push_array(self.global_variables.serialize());

        fields.storage
    }

    pub fn deserialize(serialized: [Field; HEADER_LENGTH]) -> Self {
        let mut offset = 0;

        let last_archive_fields = arr_copy_slice(serialized, [0; APPEND_ONLY_TREE_SNAPSHOT_LENGTH], offset);
        offset = offset + APPEND_ONLY_TREE_SNAPSHOT_LENGTH;

        let body_hash = arr_copy_slice(serialized, [0; NUM_FIELDS_PER_SHA256], offset);
        offset = offset + NUM_FIELDS_PER_SHA256;

        let state_fields = arr_copy_slice(serialized, [0; STATE_REFERENCE_LENGTH], offset);
        offset = offset + STATE_REFERENCE_LENGTH;

        let global_variables_fields = arr_copy_slice(serialized, [0; GLOBAL_VARIABLES_LENGTH], offset);

        Header {
            last_archive: AppendOnlyTreeSnapshot::deserialize(last_archive_fields),
            body_hash,
            state: StateReference::deserialize(state_fields),
            global_variables: GlobalVariables::deserialize(global_variables_fields),
        }
    }

    // TODO(#3941) Rename this as hash() and make it hash the whole header
    pub fn block_hash(self) -> Field {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        pedersen_hash([
            self.global_variables.hash(),
            self.state.partial.note_hash_tree.root,
            self.state.partial.nullifier_tree.root,
            self.state.partial.contract_tree.root,
            self.state.l1_to_l2_message_tree.root,
            self.state.partial.public_data_tree.root,
        ], GENERATOR_INDEX__BLOCK_HASH)
    }
}

impl Empty for Header {
    fn empty() -> Self {
        Self {
            last_archive: AppendOnlyTreeSnapshot::zero(),
            body_hash: [0; NUM_FIELDS_PER_SHA256],
            state: StateReference::empty(),
            global_variables: GlobalVariables::empty(),
        }
    }
}
