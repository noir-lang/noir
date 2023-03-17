use std::collections::BTreeMap;

use crate::CompiledProgram;

pub struct CompiledContract {
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `BTreeMap`.
    pub functions: BTreeMap<String, CompiledProgram>,
}
