use crate::{
    abis::{
    gas::Gas, note_hash::ScopedNoteHash, nullifier::ScopedNullifier,
    private_call_request::PrivateCallRequest, public_call_request::PublicCallRequest,
    log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}
},
    traits::{Serialize, Deserialize, Eq, Empty}, messaging::l2_to_l1_message::ScopedL2ToL1Message,
    utils::reader::Reader
};
use crate::constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX,
    MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX, PRIVATE_ACCUMULATED_DATA_LENGTH
};

struct PrivateAccumulatedData {
    note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],

    note_encrypted_logs_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    encrypted_logs_hashes: [ScopedEncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    unencrypted_logs_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],

    public_call_requests: [PublicCallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    private_call_stack: [PrivateCallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
}

impl Serialize<PRIVATE_ACCUMULATED_DATA_LENGTH> for PrivateAccumulatedData {
  fn serialize(self) -> [Field; PRIVATE_ACCUMULATED_DATA_LENGTH] {
    let mut fields: BoundedVec<Field, PRIVATE_ACCUMULATED_DATA_LENGTH> = BoundedVec::new();

    for i in 0..MAX_NOTE_HASHES_PER_TX {
      fields.extend_from_array(self.note_hashes[i].serialize());
    }

    for i in 0..MAX_NULLIFIERS_PER_TX {
      fields.extend_from_array(self.nullifiers[i].serialize());
    }

    for i in 0..MAX_L2_TO_L1_MSGS_PER_TX {
      fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
    }

    for i in 0..MAX_NOTE_ENCRYPTED_LOGS_PER_TX {
      fields.extend_from_array(self.note_encrypted_logs_hashes[i].serialize());
    }

    for i in 0..MAX_ENCRYPTED_LOGS_PER_TX {
      fields.extend_from_array(self.encrypted_logs_hashes[i].serialize());
    }

    for i in 0..MAX_UNENCRYPTED_LOGS_PER_TX {
      fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
    }

    for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
      fields.extend_from_array(self.public_call_requests[i].serialize());
    }

    for i in 0..MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX {
      fields.extend_from_array(self.private_call_stack[i].serialize());
    }

    assert_eq(fields.len(), PRIVATE_ACCUMULATED_DATA_LENGTH);

    fields.storage
  }
}

impl Deserialize<PRIVATE_ACCUMULATED_DATA_LENGTH> for PrivateAccumulatedData {
  fn deserialize(fields: [Field; PRIVATE_ACCUMULATED_DATA_LENGTH]) -> PrivateAccumulatedData {
    let mut reader = Reader::new(fields);

    let item = PrivateAccumulatedData {
        note_hashes: reader.read_struct_array(ScopedNoteHash::deserialize, [ScopedNoteHash::empty(); MAX_NOTE_HASHES_PER_TX]),
        nullifiers: reader.read_struct_array(ScopedNullifier::deserialize, [ScopedNullifier::empty(); MAX_NULLIFIERS_PER_TX]),
        l2_to_l1_msgs: reader.read_struct_array(ScopedL2ToL1Message::deserialize, [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX]),
        note_encrypted_logs_hashes: reader.read_struct_array(NoteLogHash::deserialize, [NoteLogHash::empty(); MAX_NOTE_ENCRYPTED_LOGS_PER_TX]),
        encrypted_logs_hashes: reader.read_struct_array(ScopedEncryptedLogHash::deserialize, [ScopedEncryptedLogHash::empty(); MAX_ENCRYPTED_LOGS_PER_TX]),
        unencrypted_logs_hashes: reader.read_struct_array(ScopedLogHash::deserialize, [ScopedLogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_TX]),
        public_call_requests: reader.read_struct_array(PublicCallRequest::deserialize, [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]),
        private_call_stack: reader.read_struct_array(PrivateCallRequest::deserialize, [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX]),
    };
    reader.finish();
    item
  }
}

impl Eq for PrivateAccumulatedData {
  fn eq(self, other: Self) -> bool {
    (self.note_hashes == other.note_hashes) &
    (self.nullifiers == other.nullifiers) &
    (self.l2_to_l1_msgs == other.l2_to_l1_msgs) &
    (self.note_encrypted_logs_hashes == other.note_encrypted_logs_hashes) &
    (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
    (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
    (self.public_call_requests == other.public_call_requests) &
    (self.private_call_stack == other.private_call_stack)
  }
}

impl Empty for PrivateAccumulatedData {
    fn empty() -> Self {
        PrivateAccumulatedData {
            note_hashes: [ScopedNoteHash::empty(); MAX_NOTE_HASHES_PER_TX],
            nullifiers: [ScopedNullifier::empty(); MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            note_encrypted_logs_hashes: [NoteLogHash::empty(); MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
            encrypted_logs_hashes: [ScopedEncryptedLogHash::empty(); MAX_ENCRYPTED_LOGS_PER_TX],
            unencrypted_logs_hashes: [ScopedLogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_TX],
            public_call_requests: [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
            private_call_stack: [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
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
