use crate::{
    abis::{
        log_hash::LogHash, private_log::PrivateLog, public_call_request::PublicCallRequest,
        side_effect::scoped::Scoped,
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};
use crate::constants::{
    MAX_CONTRACT_CLASS_LOGS_PER_TX, MAX_ENQUEUED_CALLS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_LOGS_PER_TX,
    PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH,
};

pub struct PrivateToPublicAccumulatedData {
    pub note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    pub nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    pub l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],
    pub private_logs: [PrivateLog; MAX_PRIVATE_LOGS_PER_TX],
    pub contract_class_logs_hashes: [Scoped<LogHash>; MAX_CONTRACT_CLASS_LOGS_PER_TX],
    pub public_call_requests: [PublicCallRequest; MAX_ENQUEUED_CALLS_PER_TX],
}

impl Empty for PrivateToPublicAccumulatedData {
    fn empty() -> Self {
        PrivateToPublicAccumulatedData {
            note_hashes: [0; MAX_NOTE_HASHES_PER_TX],
            nullifiers: [0; MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            private_logs: [PrivateLog::empty(); MAX_PRIVATE_LOGS_PER_TX],
            contract_class_logs_hashes: [Scoped::empty(); MAX_CONTRACT_CLASS_LOGS_PER_TX],
            public_call_requests: [PublicCallRequest::empty(); MAX_ENQUEUED_CALLS_PER_TX],
        }
    }
}

impl Eq for PrivateToPublicAccumulatedData {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
            & (self.private_logs == other.private_logs)
            & (self.contract_class_logs_hashes == other.contract_class_logs_hashes)
            & (self.public_call_requests == other.public_call_requests)
    }
}

impl Serialize<PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH> for PrivateToPublicAccumulatedData {
    fn serialize(self) -> [Field; PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH> =
            BoundedVec::new();

        fields.extend_from_array(self.note_hashes);
        fields.extend_from_array(self.nullifiers);
        for i in 0..self.l2_to_l1_msgs.len() {
            fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
        }
        for i in 0..self.private_logs.len() {
            fields.extend_from_array(self.private_logs[i].serialize());
        }
        for i in 0..self.contract_class_logs_hashes.len() {
            fields.extend_from_array(self.contract_class_logs_hashes[i].serialize());
        }
        for i in 0..self.public_call_requests.len() {
            fields.extend_from_array(self.public_call_requests[i].serialize());
        }

        assert_eq(fields.len(), PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH> for PrivateToPublicAccumulatedData {
    fn deserialize(
        fields: [Field; PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH],
    ) -> PrivateToPublicAccumulatedData {
        let mut reader = Reader::new(fields);

        let item = PrivateToPublicAccumulatedData {
            note_hashes: reader.read_array(),
            nullifiers: reader.read_array(),
            l2_to_l1_msgs: reader.read_struct_array(
                ScopedL2ToL1Message::deserialize,
                [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            ),
            private_logs: reader.read_struct_array(
                PrivateLog::deserialize,
                [PrivateLog::empty(); MAX_PRIVATE_LOGS_PER_TX],
            ),
            contract_class_logs_hashes: reader.read_struct_array(
                Scoped::deserialize,
                [Scoped::empty(); MAX_CONTRACT_CLASS_LOGS_PER_TX],
            ),
            public_call_requests: reader.read_struct_array(
                PublicCallRequest::deserialize,
                [PublicCallRequest::empty(); MAX_ENQUEUED_CALLS_PER_TX],
            ),
        };
        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty_private_to_public_accumulated_data() {
    let item = PrivateToPublicAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateToPublicAccumulatedData::deserialize(serialized);
    assert_eq(item, deserialized);
}
