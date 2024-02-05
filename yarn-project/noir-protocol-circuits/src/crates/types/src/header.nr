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
        STATE_REFERENCE_LENGTH,
    },
    hash::pedersen_hash,
    state_reference::StateReference,
    traits::{
        Deserialize,
        Empty,
        Hash,
        Serialize,
    },
    utils::{
        arr_copy_slice,
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

impl Eq for Header {
    fn eq(self, other: Self) -> bool {
        self.last_archive.eq(other.last_archive) &
        (self.body_hash == other.body_hash) &
        self.state.eq(other.state) &
        self.global_variables.eq(other.global_variables)
    }
}

impl Serialize<HEADER_LENGTH> for Header {
    fn serialize(self) -> [Field; HEADER_LENGTH] {
        let mut fields: BoundedVec<Field, HEADER_LENGTH> = BoundedVec::new(0);

        fields.extend_from_array(self.last_archive.serialize());
        fields.extend_from_array(self.body_hash);
        fields.extend_from_array(self.state.serialize());
        fields.extend_from_array(self.global_variables.serialize());

        fields.storage
    }
}

impl Deserialize<HEADER_LENGTH> for Header {
    fn deserialize(serialized: [Field; HEADER_LENGTH]) -> Self {
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

impl Hash for Header {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__BLOCK_HASH)
    }
}

#[test]
fn serialization_of_empty() {
    let header: Header = dep::std::unsafe::zeroed();
    let serialized = header.serialize();
    let deserialized = Header::deserialize(serialized);
    assert(header.eq(deserialized));
}

#[test]
fn hash_smoke() {
    let header: Header = dep::std::unsafe::zeroed();
    let _hashed = header.hash();
}
