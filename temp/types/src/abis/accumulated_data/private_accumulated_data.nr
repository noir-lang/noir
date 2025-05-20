use crate::{
    abis::{
        log_hash::LogHash,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_call_request::PrivateCallRequest,
        private_log::PrivateLogData,
        public_call_request::PublicCallRequest,
        side_effect::{Counted, scoped::Scoped},
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};
use crate::constants::{
    MAX_CONTRACT_CLASS_LOGS_PER_TX, MAX_ENQUEUED_CALLS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PRIVATE_LOGS_PER_TX, PRIVATE_ACCUMULATED_DATA_LENGTH,
};

pub struct PrivateAccumulatedData {
    pub note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    pub nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    pub l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],

    pub private_logs: [Scoped<PrivateLogData>; MAX_PRIVATE_LOGS_PER_TX],
    pub contract_class_logs_hashes: [Scoped<Counted<LogHash>>; MAX_CONTRACT_CLASS_LOGS_PER_TX],

    pub public_call_requests: [Counted<PublicCallRequest>; MAX_ENQUEUED_CALLS_PER_TX],
    pub private_call_stack: [PrivateCallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
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

        for i in 0..MAX_PRIVATE_LOGS_PER_TX {
            fields.extend_from_array(self.private_logs[i].serialize());
        }

        for i in 0..MAX_CONTRACT_CLASS_LOGS_PER_TX {
            fields.extend_from_array(self.contract_class_logs_hashes[i].serialize());
        }

        for i in 0..MAX_ENQUEUED_CALLS_PER_TX {
            fields.extend_from_array(self.public_call_requests[i].serialize());
        }

        for i in 0..MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX {
            fields.extend_from_array(self.private_call_stack[i].serialize());
        }

        assert_eq(fields.len(), PRIVATE_ACCUMULATED_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_ACCUMULATED_DATA_LENGTH> for PrivateAccumulatedData {
    fn deserialize(fields: [Field; PRIVATE_ACCUMULATED_DATA_LENGTH]) -> PrivateAccumulatedData {
        let mut reader = Reader::new(fields);

        let item = PrivateAccumulatedData {
            note_hashes: reader.read_struct_array(
                ScopedNoteHash::deserialize,
                [ScopedNoteHash::empty(); MAX_NOTE_HASHES_PER_TX],
            ),
            nullifiers: reader.read_struct_array(
                ScopedNullifier::deserialize,
                [ScopedNullifier::empty(); MAX_NULLIFIERS_PER_TX],
            ),
            l2_to_l1_msgs: reader.read_struct_array(
                ScopedL2ToL1Message::deserialize,
                [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            ),
            private_logs: reader.read_struct_array(
                Scoped::deserialize,
                [Scoped::empty(); MAX_PRIVATE_LOGS_PER_TX],
            ),
            contract_class_logs_hashes: reader.read_struct_array(
                Scoped::deserialize,
                [Scoped::empty(); MAX_CONTRACT_CLASS_LOGS_PER_TX],
            ),
            public_call_requests: reader.read_struct_array(
                Counted::deserialize,
                [Counted::empty(); MAX_ENQUEUED_CALLS_PER_TX],
            ),
            private_call_stack: reader.read_struct_array(
                PrivateCallRequest::deserialize,
                [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
            ),
        };
        reader.finish();
        item
    }
}

impl Eq for PrivateAccumulatedData {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
            & (self.private_logs == other.private_logs)
            & (self.contract_class_logs_hashes == other.contract_class_logs_hashes)
            & (self.public_call_requests == other.public_call_requests)
            & (self.private_call_stack == other.private_call_stack)
    }
}

impl Empty for PrivateAccumulatedData {
    fn empty() -> Self {
        PrivateAccumulatedData {
            note_hashes: [ScopedNoteHash::empty(); MAX_NOTE_HASHES_PER_TX],
            nullifiers: [ScopedNullifier::empty(); MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            private_logs: [Scoped::empty(); MAX_PRIVATE_LOGS_PER_TX],
            contract_class_logs_hashes: [Scoped::empty(); MAX_CONTRACT_CLASS_LOGS_PER_TX],
            public_call_requests: [Counted::empty(); MAX_ENQUEUED_CALLS_PER_TX],
            private_call_stack: [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
        }
    }
}

#[test]
fn serialization_of_empty_private_accumulated_data() {
    let item = PrivateAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
