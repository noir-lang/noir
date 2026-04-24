use std::marker::PhantomData;

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

/// Whether a memory operation reads from or writes to memory
#[derive(Clone, PartialEq, Eq, Debug, Hash, Copy)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum MemOpKind {
    Read,
    Write,
}

/// Operation on a block of memory
/// We can either write or read at an index in memory
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct MemOp<F> {
    pub operation: MemOpKind,
    /// array index, it must be less than the array length
    pub index: Witness,
    /// the witness we are reading into (read), or the witness whose value is written (write)
    pub value: Witness,
    #[cfg_attr(feature = "arb", proptest(value = "PhantomData"))]
    _phantom: PhantomData<F>,
}

/// Wire format for `MemOp` — preserves backwards-compatible serialization where all three
/// fields are `Expression<F>`. The `serde(rename)` ensures this type registers under the
/// same name ("MemOp") as the public type so that `serde_reflection` traces it correctly.
#[derive(Serialize, Deserialize)]
#[serde(rename = "MemOp")]
struct MemOpWire<F> {
    operation: Expression<F>,
    index: Expression<F>,
    value: Expression<F>,
}

impl<F: AcirField + Serialize> Serialize for MemOp<F> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        MemOpWire {
            operation: match self.operation {
                MemOpKind::Read => Expression::<F>::zero(),
                MemOpKind::Write => Expression::<F>::one(),
            },
            index: Expression::<F>::from(self.index),
            value: Expression::<F>::from(self.value),
        }
        .serialize(serializer)
    }
}

impl<'de, F: AcirField + Deserialize<'de>> Deserialize<'de> for MemOp<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = MemOpWire::<F>::deserialize(deserializer)?;

        let operation = if wire.operation.is_zero() {
            MemOpKind::Read
        } else if wire.operation.is_one() {
            MemOpKind::Write
        } else {
            return Err(serde::de::Error::custom(
                "MemOp operation must be either 0 (Read) or 1 (Write)",
            ));
        };
        let index = wire
            .index
            .to_witness()
            .ok_or_else(|| serde::de::Error::custom("MemOp index must be a single witness"))?;
        let value = wire
            .value
            .to_witness()
            .ok_or_else(|| serde::de::Error::custom("MemOp value must be a single witness"))?;

        Ok(MemOp { operation, index, value, _phantom: PhantomData })
    }
}

impl<F> MemOp<F> {
    /// Creates a `MemOp` which reads from memory at `index` and inserts the read value
    /// into the [`WitnessMap`][crate::native_types::WitnessMap] at `value`.
    pub fn read_at_mem_index(index: Witness, value: Witness) -> Self {
        MemOp { operation: MemOpKind::Read, index, value, _phantom: PhantomData }
    }

    /// Creates a `MemOp` which writes `value` into memory at `index`.
    pub fn write_to_mem_index(index: Witness, value: Witness) -> Self {
        MemOp { operation: MemOpKind::Write, index, value, _phantom: PhantomData }
    }
}
