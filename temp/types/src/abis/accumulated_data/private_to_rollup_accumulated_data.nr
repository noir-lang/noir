use crate::{
    abis::{log_hash::LogHash, private_log::PrivateLog, side_effect::scoped::Scoped},
    constants::{
        MAX_CONTRACT_CLASS_LOGS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX,
        MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_LOGS_PER_TX, PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH,
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub struct PrivateToRollupAccumulatedData {
    pub note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    pub nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    pub l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],
    pub private_logs: [PrivateLog; MAX_PRIVATE_LOGS_PER_TX],
    pub contract_class_logs_hashes: [Scoped<LogHash>; MAX_CONTRACT_CLASS_LOGS_PER_TX],
}

impl Empty for PrivateToRollupAccumulatedData {
    fn empty() -> Self {
        PrivateToRollupAccumulatedData {
            note_hashes: [0; MAX_NOTE_HASHES_PER_TX],
            nullifiers: [0; MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            private_logs: [PrivateLog::empty(); MAX_PRIVATE_LOGS_PER_TX],
            contract_class_logs_hashes: [Scoped::empty(); MAX_CONTRACT_CLASS_LOGS_PER_TX],
        }
    }
}

impl Serialize<PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH> for PrivateToRollupAccumulatedData {
    fn serialize(self) -> [Field; PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH> =
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

        assert_eq(fields.len(), PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH> for PrivateToRollupAccumulatedData {
    fn deserialize(
        fields: [Field; PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH],
    ) -> PrivateToRollupAccumulatedData {
        let mut reader = Reader::new(fields);

        let item = PrivateToRollupAccumulatedData {
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
        };
        reader.finish();
        item
    }
}

impl Eq for PrivateToRollupAccumulatedData {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
            & (self.private_logs == other.private_logs)
            & (self.contract_class_logs_hashes == other.contract_class_logs_hashes)
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateToRollupAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateToRollupAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
