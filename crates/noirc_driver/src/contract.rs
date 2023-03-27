use std::collections::BTreeMap;

use noirc_frontend::ContractVisibility;

use crate::CompiledProgram;

/// Each function in the contract will be compiled
/// as a separate noir program.
///
/// A contract function unlike a regular Noir program
/// however can have addition properties.
/// One of these being a function type.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ContractFunction {
    pub func_type: ContractVisibility,
    pub function: CompiledProgram,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CompiledContract {
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `BTreeMap`.
    pub functions: BTreeMap<String, ContractFunction>,
}
