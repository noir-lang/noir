use crate::program::{deserialize_circuit, serialize_circuit};
use acvm::acir::circuit::Circuit;
use noirc_abi::Abi;
use serde::{Deserialize, Serialize};

/// Describes the types of smart contract functions that are allowed.
/// Unlike the similar enum in noirc_frontend, 'open' and 'unconstrained'
/// are mutually exclusive here. In the case a function is both, 'unconstrained'
/// takes precedence.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize)]
pub struct CompiledContract {
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `Vector`.
    pub functions: Vec<ContractFunction>,
}

/// Each function in the contract will be compiled
/// as a separate noir program.
///
/// A contract function unlike a regular Noir program
/// however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractFunction {
    pub name: String,

    pub function_type: ContractFunctionType,

    pub abi: Abi,

    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub bytecode: Circuit,
}

impl ContractFunctionType {
    pub(super) fn new(kind: noirc_frontend::ContractFunctionType, is_unconstrained: bool) -> Self {
        match (kind, is_unconstrained) {
            (_, true) => Self::Unconstrained,
            (noirc_frontend::ContractFunctionType::Secret, false) => Self::Secret,
            (noirc_frontend::ContractFunctionType::Open, false) => Self::Open,
        }
    }
}
