use noirc_errors::Location;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
/// Represents whether a parameter or return value is public or known only to the prover.
pub enum Visibility {
    Public(Location),
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    #[default]
    Private,
    /// DataBus is public input handled as private input. We use the fact that return values are properly computed by the program to avoid having them as public inputs
    /// it is useful for recursion and is handled by the proving system.
    /// The u32 value is used to group inputs having the same value.
    CallData(u32, Location),
    ReturnData(Location),
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public(_) => write!(f, "pub"),
            Self::Private => write!(f, ""),
            Self::CallData(id, _) => write!(f, "call_data({id})"),
            Self::ReturnData(_) => write!(f, "return_data"),
        }
    }
}
