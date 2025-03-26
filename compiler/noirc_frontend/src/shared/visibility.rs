use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
/// Represents whether the parameter is public or known only to the prover.
pub enum Visibility {
    Public,
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    #[default]
    Private,
    /// DataBus is public input handled as private input. We use the fact that return values are properly computed by the program to avoid having them as public inputs
    /// it is useful for recursion and is handled by the proving system.
    /// The u32 value is used to group inputs having the same value.
    CallData(u32),
    ReturnData,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "pub"),
            Self::Private => write!(f, "priv"),
            Self::CallData(id) => write!(f, "calldata{id}"),
            Self::ReturnData => write!(f, "returndata"),
        }
    }
}
