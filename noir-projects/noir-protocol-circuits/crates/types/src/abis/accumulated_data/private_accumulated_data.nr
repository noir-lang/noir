use crate::{
    abis::{
    call_request::CallRequest, gas::Gas, note_hash::ScopedNoteHash, nullifier::ScopedNullifier,
    side_effect::SideEffect
},
    traits::{Serialize, Deserialize, Eq, Empty}, messaging::l2_to_l1_message::ScopedL2ToL1Message,
    utils::reader::Reader
};
use crate::constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX,
    MAX_UNENCRYPTED_LOGS_PER_TX, PRIVATE_ACCUMULATED_DATA_LENGTH
};

struct PrivateAccumulatedData {
    new_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
    new_l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_TX],
    unencrypted_logs_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_TX],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
}

impl Serialize<PRIVATE_ACCUMULATED_DATA_LENGTH> for PrivateAccumulatedData {
  fn serialize(self) -> [Field; PRIVATE_ACCUMULATED_DATA_LENGTH] {
    let mut fields: BoundedVec<Field, PRIVATE_ACCUMULATED_DATA_LENGTH> = BoundedVec::new();

    for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
      fields.extend_from_array(self.new_note_hashes[i].serialize());
    }

    for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
      fields.extend_from_array(self.new_nullifiers[i].serialize());
    }

    for i in 0..MAX_NEW_L2_TO_L1_MSGS_PER_TX {
      fields.extend_from_array(self.new_l2_to_l1_msgs[i].serialize());
    }

    for i in 0..MAX_ENCRYPTED_LOGS_PER_TX {
      fields.extend_from_array(self.encrypted_logs_hashes[i].serialize());
    }

    for i in 0..MAX_UNENCRYPTED_LOGS_PER_TX {
      fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
    }

    fields.push(self.encrypted_log_preimages_length);
    fields.push(self.unencrypted_log_preimages_length);

    for i in 0..MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX {
      fields.extend_from_array(self.private_call_stack[i].serialize());
    }

    for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
      fields.extend_from_array(self.public_call_stack[i].serialize());
    }

    assert_eq(fields.len(), PRIVATE_ACCUMULATED_DATA_LENGTH);

    fields.storage
  }
}

impl Deserialize<PRIVATE_ACCUMULATED_DATA_LENGTH> for PrivateAccumulatedData {
  fn deserialize(fields: [Field; PRIVATE_ACCUMULATED_DATA_LENGTH]) -> PrivateAccumulatedData {
    let mut reader = Reader::new(fields);

    let item = PrivateAccumulatedData {
        new_note_hashes: reader.read_struct_array(ScopedNoteHash::deserialize, [ScopedNoteHash::empty(); MAX_NEW_NOTE_HASHES_PER_TX]),
        new_nullifiers: reader.read_struct_array(ScopedNullifier::deserialize, [ScopedNullifier::empty(); MAX_NEW_NULLIFIERS_PER_TX]),
        new_l2_to_l1_msgs: reader.read_struct_array(ScopedL2ToL1Message::deserialize, [ScopedL2ToL1Message::empty(); MAX_NEW_L2_TO_L1_MSGS_PER_TX]),
        encrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_TX]),
        unencrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_TX]),
        encrypted_log_preimages_length: reader.read(),
        unencrypted_log_preimages_length: reader.read(),
        private_call_stack: reader.read_struct_array(CallRequest::deserialize, [CallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX]),
        public_call_stack: reader.read_struct_array(CallRequest::deserialize, [CallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]),
    };
    reader.finish();
    item
  }
}

impl Eq for PrivateAccumulatedData {
  fn eq(self, other: Self) -> bool {
    (self.new_note_hashes == other.new_note_hashes) &
    (self.new_nullifiers == other.new_nullifiers) &
    (self.new_l2_to_l1_msgs == other.new_l2_to_l1_msgs) &
    (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
    (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
    (self.encrypted_log_preimages_length == other.encrypted_log_preimages_length) &
    (self.unencrypted_log_preimages_length == other.unencrypted_log_preimages_length) &
    (self.private_call_stack == other.private_call_stack) &
    (self.public_call_stack == other.public_call_stack)
  }
}

impl Empty for PrivateAccumulatedData {
    fn empty() -> Self {
        PrivateAccumulatedData {
            new_note_hashes: [ScopedNoteHash::empty(); MAX_NEW_NOTE_HASHES_PER_TX],
            new_nullifiers: [ScopedNullifier::empty(); MAX_NEW_NULLIFIERS_PER_TX],
            new_l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_NEW_L2_TO_L1_MSGS_PER_TX],
            encrypted_logs_hashes: [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_TX],
            unencrypted_logs_hashes: [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_TX],
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            private_call_stack: [CallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
            public_call_stack: [CallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
        }
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
