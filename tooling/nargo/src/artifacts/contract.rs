use acvm::acir::circuit::Circuit;
use noirc_abi::{Abi, ContractEvent};
use noirc_driver::{CompiledContract, ContractFunction, ContractFunctionType};
use serde::{Deserialize, Serialize};

use noirc_driver::DebugFile;
use noirc_errors::debug_info::DebugInfo;
use std::collections::BTreeMap;

use fm::FileId;

#[derive(Serialize, Deserialize)]
pub struct ContractArtifact {
    /// Version of noir used to compile this contract
    pub noir_version: String,
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate program stored in this `Vec`.
    pub functions: Vec<ContractFunctionArtifact>,
    /// All the events defined inside the contract scope.
    pub events: Vec<ContractEvent>,
    /// Map of file Id to the source code so locations in debug info can be mapped to source code they point to.
    pub file_map: BTreeMap<FileId, DebugFile>,
}

impl From<CompiledContract> for ContractArtifact {
    fn from(contract: CompiledContract) -> Self {
        ContractArtifact {
            noir_version: contract.noir_version,
            name: contract.name,
            functions: contract.functions.into_iter().map(ContractFunctionArtifact::from).collect(),
            events: contract.events,
            file_map: contract.file_map,
        }
    }
}

/// Each function in the contract will be compiled as a separate noir program.
///
/// A contract function unlike a regular Noir program however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractFunctionArtifact {
    pub name: String,

    pub function_type: ContractFunctionType,

    pub is_internal: bool,

    pub abi: Abi,

    #[serde(
        serialize_with = "Circuit::serialize_circuit_base64",
        deserialize_with = "Circuit::deserialize_circuit_base64"
    )]
    pub bytecode: Circuit,

    #[serde(
        serialize_with = "DebugInfo::serialize_compressed_base64_json",
        deserialize_with = "DebugInfo::deserialize_compressed_base64_json"
    )]
    pub debug_symbols: DebugInfo,
}

impl From<ContractFunction> for ContractFunctionArtifact {
    fn from(func: ContractFunction) -> Self {
        ContractFunctionArtifact {
            name: func.name,
            function_type: func.function_type,
            is_internal: func.is_internal,
            abi: func.abi,
            bytecode: func.bytecode,
            debug_symbols: func.debug,
        }
    }
}
