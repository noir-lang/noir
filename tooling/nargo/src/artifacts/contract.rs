use acvm::acir::circuit::Circuit;
use noirc_abi::{Abi, ContractEvent};
use noirc_driver::{ContractFunction, ContractFunctionType};
use serde::{Deserialize, Serialize};

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
}

impl From<ContractFunction> for ContractFunctionArtifact {
    fn from(func: ContractFunction) -> Self {
        ContractFunctionArtifact {
            name: func.name,
            function_type: func.function_type,
            is_internal: func.is_internal,
            abi: func.abi,
            bytecode: func.bytecode,
        }
    }
}
