use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use acvm::{acir::circuit::Program, FieldElement};
use fm::FileId;
use noirc_abi::{Abi, AbiType, AbiValue};
use noirc_errors::debug_info::DebugInfo;
use noirc_evaluator::errors::SsaReport;

use super::debug::DebugFile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledContractOutputs {
    pub structs: HashMap<String, Vec<AbiType>>,
    pub globals: HashMap<String, Vec<AbiValue>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledContract {
    pub noir_version: String,

    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `Vector`.
    pub functions: Vec<ContractFunction>,

    pub outputs: CompiledContractOutputs,

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

    pub is_unconstrained: bool,

    pub custom_attributes: Vec<String>,

    pub abi: Abi,

    #[serde(
        serialize_with = "Program::serialize_program_base64",
        deserialize_with = "Program::deserialize_program_base64"
    )]
    pub bytecode: Program<FieldElement>,

    pub debug: Vec<DebugInfo>,

    /// Names of the functions in the program. These are used for more informative debugging and benchmarking.
    pub names: Vec<String>,
}
