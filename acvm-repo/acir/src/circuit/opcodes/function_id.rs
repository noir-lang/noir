use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
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
