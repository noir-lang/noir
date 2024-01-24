use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use acvm::acir::circuit::Circuit;
use fm::FileId;
use noirc_abi::{Abi, ContractEvent};
use noirc_errors::debug_info::DebugInfo;
use noirc_evaluator::errors::SsaReport;

use super::debug::DebugFile;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledContract {
    pub noir_version: String,

    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `Vector`.
    pub functions: Vec<ContractFunction>,

    /// All the events defined inside the contract scope.
    /// An event is a struct value that can be emitted via oracles
    /// by any contract function during execution.
    pub events: Vec<ContractEvent>,

    pub file_map: BTreeMap<FileId, DebugFile>,
    pub warnings: Vec<SsaReport>,
}

/// Each function in the contract will be compiled
/// as a separate noir program.
///
/// A contract function unlike a regular Noir program
/// however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFunction {
    pub name: String,

    pub function_type: ContractFunctionType,

    pub is_internal: bool,

    pub abi: Abi,

    #[serde(
        serialize_with = "Circuit::serialize_circuit_base64",
        deserialize_with = "Circuit::deserialize_circuit_base64"
    )]
    pub bytecode: Circuit,

    pub debug: DebugInfo,
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
