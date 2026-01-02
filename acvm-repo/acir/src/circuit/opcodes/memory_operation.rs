use crate::native_types::{Expression, Witness};
use acir_field::AcirField;
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
pub struct MemOp<F> {
    /// A constant expression that can be 0 (read) or 1 (write)
    pub operation: Expression<F>,
    /// array index, it must be less than the array length
    pub index: Expression<F>,
    /// the value we are reading, when operation is 0, or the value we write at
    /// the specified index, when operation is 1
    pub value: Expression<F>,
}

impl<F: AcirField> MemOp<F> {
    /// Creates a `MemOp` which reads from memory at `index` and inserts the read value
    /// into the [`WitnessMap`][crate::native_types::WitnessMap] at `witness`
    pub fn read_at_mem_index(index: Expression<F>, witness: Witness) -> Self {
        MemOp { operation: Expression::zero(), index, value: witness.into() }
    }

    /// Creates a `MemOp` which writes the [`Expression`] `value` into memory at `index`.
    pub fn write_to_mem_index(index: Expression<F>, value: Expression<F>) -> Self {
        MemOp { operation: Expression::one(), index, value }
    }
}
