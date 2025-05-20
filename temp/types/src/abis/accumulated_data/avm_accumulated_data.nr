use crate::{
    abis::{public_data_write::PublicDataWrite, public_log::PublicLog},
    constants::{
        AVM_ACCUMULATED_DATA_LENGTH, MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX,
        MAX_NULLIFIERS_PER_TX, MAX_PUBLIC_LOGS_PER_TX, MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub struct AvmAccumulatedData {
    // The note hashes from private combining with those made in the AVM execution.
    pub note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    // The nullifiers from private combining with those made in the AVM execution.
    pub nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    // The L2 to L1 messages from private combining with those made in the AVM execution.
    pub l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],
    // The public logs emitted from the AVM execution.
    pub public_logs: [PublicLog; MAX_PUBLIC_LOGS_PER_TX],
    // The public data writes made in the AVM execution.
    pub public_data_writes: [PublicDataWrite; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
}

impl Empty for AvmAccumulatedData {
    fn empty() -> Self {
        AvmAccumulatedData {
            note_hashes: [0; MAX_NOTE_HASHES_PER_TX],
            nullifiers: [0; MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            public_logs: [PublicLog::empty(); MAX_PUBLIC_LOGS_PER_TX],
            public_data_writes: [
                PublicDataWrite::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
            ],
        }
    }
}

impl Serialize<AVM_ACCUMULATED_DATA_LENGTH> for AvmAccumulatedData {
    fn serialize(self) -> [Field; AVM_ACCUMULATED_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, AVM_ACCUMULATED_DATA_LENGTH> = BoundedVec::new();

        for i in 0..self.note_hashes.len() {
            fields.extend_from_array(self.note_hashes[i].serialize());
        }

        for i in 0..self.nullifiers.len() {
            fields.extend_from_array(self.nullifiers[i].serialize());
        }

        for i in 0..self.l2_to_l1_msgs.len() {
            fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
        }

        for i in 0..self.public_logs.len() {
            fields.extend_from_array(self.public_logs[i].serialize());
        }

        for i in 0..self.public_data_writes.len() {
            fields.extend_from_array(self.public_data_writes[i].serialize());
        }

        assert_eq(fields.len(), AVM_ACCUMULATED_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<AVM_ACCUMULATED_DATA_LENGTH> for AvmAccumulatedData {
    fn deserialize(fields: [Field; AVM_ACCUMULATED_DATA_LENGTH]) -> AvmAccumulatedData {
        let mut reader = Reader::new(fields);

        let item = AvmAccumulatedData {
            note_hashes: reader.read_array(),
            nullifiers: reader.read_array(),
            l2_to_l1_msgs: reader.read_struct_array(
                ScopedL2ToL1Message::deserialize,
                [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            ),
            public_logs: reader.read_struct_array(
                PublicLog::deserialize,
                [PublicLog::empty(); MAX_PUBLIC_LOGS_PER_TX],
            ),
            public_data_writes: reader.read_struct_array(
                PublicDataWrite::deserialize,
                [PublicDataWrite::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
            ),
        };
        reader.finish();
        item
    }
}

impl Eq for AvmAccumulatedData {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
            & (self.public_logs == other.public_logs)
            & (self.public_data_writes == other.public_data_writes)
    }
}

#[test]
fn serialization_of_empty_avm_accumulated_data() {
    let item = AvmAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = AvmAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
