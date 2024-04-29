use crate::{
    abis::side_effect::{Ordered, OrderedValue}, address::AztecAddress, constants::NULLIFIER_LENGTH,
    traits::{Empty, Hash, Serialize, Deserialize}
};

struct Nullifier {
    value: Field,
    counter: u32,
    note_hash: Field,
}

impl Ordered for Nullifier {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl OrderedValue<Field> for Nullifier {
    fn value(self) -> Field {
        self.value
    }
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for Nullifier {
    fn eq(self, other: Nullifier) -> bool {
        (self.value == other.value)
            & (self.counter == other.counter)
            & (self.note_hash == other.note_hash) 
    }
}

impl Empty for Nullifier {
    fn empty() -> Self {
        Nullifier {
            value: 0,
            counter: 0,
            note_hash: 0,
        }
    }
}

impl Serialize<NULLIFIER_LENGTH> for Nullifier {
    fn serialize(self) -> [Field; NULLIFIER_LENGTH] {
        [self.value, self.counter as Field, self.note_hash]
    }
}

impl Deserialize<NULLIFIER_LENGTH> for Nullifier {
    fn deserialize(values: [Field; NULLIFIER_LENGTH]) -> Self {
        Self {
            value: values[0],
            counter: values[1] as u32,
            note_hash: values[2],
        }
    }
}
