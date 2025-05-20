use crate::{constants::PUBLIC_DATA_WRITE_LENGTH, traits::{Deserialize, Empty, Serialize}};

pub struct PublicDataWrite {
    pub leaf_slot: Field,
    pub value: Field,
}

impl Eq for PublicDataWrite {
    fn eq(self, other: Self) -> bool {
        (self.leaf_slot == other.leaf_slot) & (self.value == other.value)
    }
}

impl Empty for PublicDataWrite {
    fn empty() -> Self {
        Self { leaf_slot: 0, value: 0 }
    }
}

impl Serialize<PUBLIC_DATA_WRITE_LENGTH> for PublicDataWrite {
    fn serialize(self) -> [Field; PUBLIC_DATA_WRITE_LENGTH] {
        [self.leaf_slot, self.value]
    }
}

impl Deserialize<PUBLIC_DATA_WRITE_LENGTH> for PublicDataWrite {
    fn deserialize(fields: [Field; PUBLIC_DATA_WRITE_LENGTH]) -> PublicDataWrite {
        PublicDataWrite { leaf_slot: fields[0], value: fields[1] }
    }
}

#[test]
fn serialization_of_empty_public_data_write() {
    let item = PublicDataWrite::empty();
    let serialized = item.serialize();
    let deserialized = PublicDataWrite::deserialize(serialized);
    assert(item.eq(deserialized));
}
