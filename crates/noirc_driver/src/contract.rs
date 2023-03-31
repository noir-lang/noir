use std::collections::BTreeMap;

use crate::CompiledProgram;

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

/// Describes the types of smart contract functions that are allowed.
/// Unlike the similar enum in noirc_frontend, 'open' and 'unconstrained'
/// are mutually exclusive here. In the case a function is both, 'unconstrained'
/// takes precedence.
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractFunctionType {
    /// This function will be executed in a private
    /// context.
    Secret,
    /// This function will be executed in a public
    /// context.
    Open,
    /// This function cannot constrain any values and can use nondeterministic features
    /// like arrays of a dynamic size.
    Unconstrained,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CompiledContract {
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `BTreeMap`.
    pub functions: BTreeMap<String, ContractFunction>,
}

impl ContractFunctionType {
    pub fn new(kind: noirc_frontend::ContractFunctionType, is_unconstrained: bool) -> Self {
        match (kind, is_unconstrained) {
            (_, true) => Self::Unconstrained,
            (noirc_frontend::ContractFunctionType::Secret, false) => Self::Secret,
            (noirc_frontend::ContractFunctionType::Open, false) => Self::Open,
        }
    }
}
