use crate::native_types::{Expression, Witness};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default)]
pub struct BlockId(pub u32);

/// Operation on a block of memory
/// We can either write or read at an index in memory
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct MemOp {
    /// Can be 0 (read) or 1 (write)
    pub operation: Expression,
    pub index: Expression,
    pub value: Expression,
}

impl MemOp {
    /// Creates a `MemOp` which reads from memory at `index` and inserts the read value
    /// into the [`WitnessMap`][crate::native_types::WitnessMap] at `witness`
    pub fn read_at_mem_index(index: Expression, witness: Witness) -> Self {
        MemOp { operation: Expression::zero(), index, value: witness.into() }
    }

    /// Creates a `MemOp` which writes the [`Expression`] `value` into memory at `index`.
    pub fn write_to_mem_index(index: Expression, value: Expression) -> Self {
        MemOp { operation: Expression::one(), index, value }
    }
}
