use std::collections::BTreeMap;

use crate::CompiledProgram;

/// ContractFunctionType describes the types
/// smart contract functions that are allowed.
///
/// Note:
/// - All Noir programs in the non-contract context
///   can be seen as `Secret`.
/// - It may be possible to have `unconstrained`
/// functions in regular Noir programs. For now
/// we leave it as a property of only contract functions.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractFunctionType {
    /// This function will be executed in a private
    /// context.
    Secret,
    /// This function will be executed in a public
    /// context.
    Public,
    // / A function which is non-deterministic
    // / and does not require any constraint.
    Unconstrained,
}
/// Each function in the contract will be compiled
/// as a separate noir program.
///
/// A contract function unlike a regular Noir program
/// however can have addition properties.
/// One of these being a function type.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ContractFunction {
    pub func_type: ContractFunctionType,
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
