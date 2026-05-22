use crate::native_types::Witness;
use msgpack_tagged::MsgpackTagged;
use serde::{Deserialize, Serialize};

/// Identifier for a block of memory
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default)]
#[derive(Serialize, Deserialize, MsgpackTagged)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct BlockId(pub u32);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.0)
    }
}

/// Whether a memory operation reads from or writes to memory
#[derive(Clone, PartialEq, Eq, Debug, Hash, Copy)]
#[derive(MsgpackTagged)]
#[tagged(via(bool))]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum MemOpKind {
    Read,
    Write,
}

impl Serialize for MemOpKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(matches!(self, MemOpKind::Write))
    }
}

impl<'de> Deserialize<'de> for MemOpKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(if bool::deserialize(deserializer)? { MemOpKind::Write } else { MemOpKind::Read })
    }
}

/// Operation on a block of memory
/// We can either write or read at an index in memory
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[derive(Serialize, Deserialize, MsgpackTagged)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct MemOp {
    #[serde(rename = "read")]
    #[tag(0)]
    pub operation: MemOpKind,
    /// array index, it must be less than the array length
    #[tag(1)]
    pub index: Witness,
    /// the witness we are reading into (read), or the witness whose value is written (write)
    #[tag(2)]
    pub value: Witness,
}

impl MemOp {
    /// Creates a `MemOp` which reads from memory at `index` and inserts the read value
    /// into the [`WitnessMap`][crate::native_types::WitnessMap] at `value`.
    pub fn read_at_mem_index(index: Witness, value: Witness) -> Self {
        MemOp { operation: MemOpKind::Read, index, value }
    }

    /// Creates a `MemOp` which writes `value` into memory at `index`.
    pub fn write_to_mem_index(index: Witness, value: Witness) -> Self {
        MemOp { operation: MemOpKind::Write, index, value }
    }
}
