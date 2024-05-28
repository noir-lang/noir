use serde::{Deserialize, Serialize};

// Witness might be a misnomer. This is an index that represents the position a witness will take
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct Witness(pub u32);

impl Witness {
    pub fn new(witness_index: u32) -> Witness {
        Witness(witness_index)
    }
    pub fn witness_index(&self) -> u32 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        // This is safe as long as the architecture is 32bits minimum
        self.0 as usize
    }

    pub const fn can_defer_constraint(&self) -> bool {
        true
    }
}

impl From<u32> for Witness {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
