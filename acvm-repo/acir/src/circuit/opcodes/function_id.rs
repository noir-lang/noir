use msgpack_tagged::MsgpackTagged;
use serde::{Deserialize, Serialize};

/// Id for the function being called.
/// Indexes into the table of ACIR function's specified in a [program][crate::circuit::Program]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[derive(Serialize, Deserialize, MsgpackTagged)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
#[serde(transparent)]
pub struct AcirFunctionId(u32);

impl AcirFunctionId {
    /// Creates an `AcirFunctionId` indexing into a [program][crate::circuit::Program]'s
    /// table of ACIR functions by its raw index.
    pub fn new(id: u32) -> Self {
        AcirFunctionId(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for AcirFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
