use crate::native_types::Witness;
use serde::{Deserialize, Serialize};

/// Identifier for a block of memory
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct BlockId(pub u32);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.0)
    }
}

/// Operation on a block of memory
/// We can either write or read at an index in memory
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct MemOp {
    /// A constant expression that can be 0 (read) or 1 (write)
    pub operation: bool,
    /// array index, it must be less than the array length
    pub index: Witness,
    /// the value we are reading, when operation is 0, or the value we write at
    /// the specified index, when operation is 1
    pub value: Witness,
}

impl MemOp {
    /// Creates a `MemOp` which reads from memory at `index` and inserts the read value
    /// into the [`WitnessMap`][crate::native_types::WitnessMap] at `witness`
    pub fn read_at_mem_index(index: Witness, witness: Witness) -> Self {
        MemOp { operation: false, index, value: witness }
    }

    /// Creates a `MemOp` which writes the value assigned to the [`Witness`] `value` into memory at `index`.
    pub fn write_to_mem_index(index: Witness, value: Witness) -> Self {
        MemOp { operation: true, index, value }
    }
}
