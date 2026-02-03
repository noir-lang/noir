use serde::{Deserialize, Serialize};

/// Id for the function being called.
/// Indexes into the table of ACIR function's specified in a [program][crate::circuit::Program]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
#[serde(transparent)]
pub struct AcirFunctionId(pub u32);

impl AcirFunctionId {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for AcirFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
