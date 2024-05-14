use crate::{
    abis::{
    call_request::CallRequest, public_data_update_request::PublicDataUpdateRequest, gas::Gas,
    note_hash::NoteHash, nullifier::Nullifier, side_effect::SideEffect
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_NEW_L2_TO_L1_MSGS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX,
    MAX_UNENCRYPTED_LOGS_PER_TX, PUBLIC_ACCUMULATED_DATA_LENGTH
},
    traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};

struct PublicAccumulatedData {
    new_note_hashes: [NoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [Nullifier; MAX_NEW_NULLIFIERS_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_TX],
    unencrypted_logs_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_TX],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],

    gas_used: Gas,
}

impl Empty for PublicAccumulatedData {
    fn empty() -> Self {
        PublicAccumulatedData {
            new_note_hashes: [NoteHash::empty(); MAX_NEW_NOTE_HASHES_PER_TX],
            new_nullifiers: [Nullifier::empty(); MAX_NEW_NULLIFIERS_PER_TX],
            new_l2_to_l1_msgs: [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX],
            encrypted_logs_hashes: [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_TX],
            unencrypted_logs_hashes: [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_TX],
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            public_data_update_requests: [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
            public_call_stack: [CallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
            gas_used: Gas::empty(),
        }
    }
}

impl Serialize<PUBLIC_ACCUMULATED_DATA_LENGTH> for PublicAccumulatedData {
    fn serialize(self) -> [Field; PUBLIC_ACCUMULATED_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_ACCUMULATED_DATA_LENGTH> = BoundedVec::new();

        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
          fields.extend_from_array(self.new_note_hashes[i].serialize());
        }

        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
          fields.extend_from_array(self.new_nullifiers[i].serialize());
        }

        fields.extend_from_array(self.new_l2_to_l1_msgs);

        for i in 0..MAX_ENCRYPTED_LOGS_PER_TX {
          fields.extend_from_array(self.encrypted_logs_hashes[i].serialize());
        }

        for i in 0..MAX_UNENCRYPTED_LOGS_PER_TX {
          fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
        }

        fields.push(self.encrypted_log_preimages_length);
        fields.push(self.unencrypted_log_preimages_length);

        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
          fields.extend_from_array(self.public_data_update_requests[i].serialize());
        }

        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
          fields.extend_from_array(self.public_call_stack[i].serialize());
        }

        fields.extend_from_array(self.gas_used.serialize());

        assert_eq(fields.len(), PUBLIC_ACCUMULATED_DATA_LENGTH);

        fields.storage
    }
}

impl Deserialize<PUBLIC_ACCUMULATED_DATA_LENGTH> for PublicAccumulatedData {
  fn deserialize(fields: [Field; PUBLIC_ACCUMULATED_DATA_LENGTH]) -> PublicAccumulatedData {
    let mut reader = Reader::new(fields);

    let item = PublicAccumulatedData {
        new_note_hashes: reader.read_struct_array(NoteHash::deserialize, [NoteHash::empty(); MAX_NEW_NOTE_HASHES_PER_TX]),
        new_nullifiers: reader.read_struct_array(Nullifier::deserialize, [Nullifier::empty(); MAX_NEW_NULLIFIERS_PER_TX]),
        new_l2_to_l1_msgs: reader.read_array([0; MAX_NEW_L2_TO_L1_MSGS_PER_TX]),
        encrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_TX]),
        unencrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_TX]),
        encrypted_log_preimages_length: reader.read(),
        unencrypted_log_preimages_length: reader.read(),
        public_data_update_requests: reader.read_struct_array(PublicDataUpdateRequest::deserialize, [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]),
        public_call_stack: reader.read_struct_array(CallRequest::deserialize, [CallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]),
        gas_used: reader.read_struct(Gas::deserialize),
    };
    reader.finish();
    item
  }
}

impl Eq for PublicAccumulatedData {
  fn eq(self, other: Self) -> bool {
    (self.new_note_hashes == other.new_note_hashes) &
    (self.new_nullifiers == other.new_nullifiers) &
    (self.new_l2_to_l1_msgs == other.new_l2_to_l1_msgs) &
    (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
    (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
    (self.encrypted_log_preimages_length == other.encrypted_log_preimages_length) &
    (self.unencrypted_log_preimages_length == other.unencrypted_log_preimages_length) &
    (self.public_data_update_requests == other.public_data_update_requests) &
    (self.public_call_stack == other.public_call_stack) &
    (self.gas_used == other.gas_used)
  }
}

#[test]
fn serialization_of_empty() {
    let item = PublicAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PublicAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
