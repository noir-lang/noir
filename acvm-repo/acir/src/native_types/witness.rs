use serde::{Deserialize, Serialize};

/// An index that represents the position a witness value will take
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
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
}

impl From<u32> for Witness {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}
