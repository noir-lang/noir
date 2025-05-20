use crate::constants::{
    MAX_L2_TO_L1_MSGS_PER_TX, MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX,
    NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS, PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH,
};
use crate::{
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_length, reader::Reader},
};

pub struct PrivateToAvmAccumulatedData {
    pub note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    pub nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    pub l2_to_l1_msgs: [ScopedL2ToL1Message; MAX_L2_TO_L1_MSGS_PER_TX],
}

impl Empty for PrivateToAvmAccumulatedData {
    fn empty() -> Self {
        PrivateToAvmAccumulatedData {
            note_hashes: [0; MAX_NOTE_HASHES_PER_TX],
            nullifiers: [0; MAX_NULLIFIERS_PER_TX],
            l2_to_l1_msgs: [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
        }
    }
}

impl Eq for PrivateToAvmAccumulatedData {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
    }
}

impl Serialize<PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH> for PrivateToAvmAccumulatedData {
    fn serialize(self) -> [Field; PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH> =
            BoundedVec::new();

        fields.extend_from_array(self.note_hashes);
        fields.extend_from_array(self.nullifiers);
        for i in 0..self.l2_to_l1_msgs.len() {
            fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
        }

        assert_eq(fields.len(), PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH> for PrivateToAvmAccumulatedData {
    fn deserialize(
        fields: [Field; PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH],
    ) -> PrivateToAvmAccumulatedData {
        let mut reader = Reader::new(fields);

        let item = PrivateToAvmAccumulatedData {
            note_hashes: reader.read_array(),
            nullifiers: reader.read_array(),
            l2_to_l1_msgs: reader.read_struct_array(
                ScopedL2ToL1Message::deserialize,
                [ScopedL2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_TX],
            ),
        };
        reader.finish();
        item
    }
}

pub struct PrivateToAvmAccumulatedDataArrayLengths {
    pub note_hashes: u32,
    pub nullifiers: u32,
    pub l2_to_l1_msgs: u32,
}

impl PrivateToAvmAccumulatedDataArrayLengths {
    pub fn new(data: PrivateToAvmAccumulatedData) -> Self {
        PrivateToAvmAccumulatedDataArrayLengths {
            note_hashes: array_length(data.note_hashes),
            nullifiers: array_length(data.nullifiers),
            l2_to_l1_msgs: array_length(data.l2_to_l1_msgs),
        }
    }
}

impl Empty for PrivateToAvmAccumulatedDataArrayLengths {
    fn empty() -> Self {
        PrivateToAvmAccumulatedDataArrayLengths { note_hashes: 0, nullifiers: 0, l2_to_l1_msgs: 0 }
    }
}

impl Eq for PrivateToAvmAccumulatedDataArrayLengths {
    fn eq(self, other: Self) -> bool {
        (self.note_hashes == other.note_hashes)
            & (self.nullifiers == other.nullifiers)
            & (self.l2_to_l1_msgs == other.l2_to_l1_msgs)
    }
}

impl Serialize<NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS> for PrivateToAvmAccumulatedDataArrayLengths {
    fn serialize(self) -> [Field; NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS] {
        let mut fields: BoundedVec<Field, NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS> =
            BoundedVec::new();

        fields.push(self.note_hashes as Field);
        fields.push(self.nullifiers as Field);
        fields.push(self.l2_to_l1_msgs as Field);

        fields.storage()
    }
}

impl Deserialize<NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS> for PrivateToAvmAccumulatedDataArrayLengths {
    fn deserialize(
        fields: [Field; NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS],
    ) -> PrivateToAvmAccumulatedDataArrayLengths {
        let mut reader = Reader::new(fields);

        let item = Self {
            note_hashes: reader.read_u32(),
            nullifiers: reader.read_u32(),
            l2_to_l1_msgs: reader.read_u32(),
        };

        reader.finish();
        item
    }
}

#[test]
fn serialization_of_empty_private_to_public_accumulated_data() {
    let item = PrivateToAvmAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = PrivateToAvmAccumulatedData::deserialize(serialized);
    assert_eq(item, deserialized);
}
