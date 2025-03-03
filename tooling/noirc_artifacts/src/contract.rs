use acvm::{FieldElement, acir::circuit::Program};
use noirc_abi::{Abi, AbiType, AbiValue};
use noirc_driver::{CompiledContract, CompiledContractOutputs, CompiledProgram, ContractFunction};
use serde::{Deserialize, Serialize};

use noirc_driver::DebugFile;
use noirc_errors::debug_info::ProgramDebugInfo;
use std::collections::{BTreeMap, HashMap};

use fm::FileId;

use super::{deserialize_hash, serialize_hash};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ContractOutputsArtifact {
    pub structs: HashMap<String, Vec<AbiType>>,
    pub globals: HashMap<String, Vec<AbiValue>>,
}

impl From<CompiledContractOutputs> for ContractOutputsArtifact {
    fn from(outputs: CompiledContractOutputs) -> Self {
        ContractOutputsArtifact { structs: outputs.structs, globals: outputs.globals }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ContractArtifact {
    /// Version of noir used to compile this contract
    pub noir_version: String,
    /// The name of the contract.
    pub name: String,
    /// Each of the contract's functions are compiled into a separate program stored in this `Vec`.
    pub functions: Vec<ContractFunctionArtifact>,

    pub outputs: ContractOutputsArtifact,
    /// Map of file Id to the source code so locations in debug info can be mapped to source code they point to.
    pub file_map: BTreeMap<FileId, DebugFile>,
}

impl From<CompiledContract> for ContractArtifact {
    fn from(contract: CompiledContract) -> Self {
        ContractArtifact {
            noir_version: contract.noir_version,
            name: contract.name,
            functions: contract.functions.into_iter().map(ContractFunctionArtifact::from).collect(),
            outputs: contract.outputs.into(),
            file_map: contract.file_map,
        }
    }
}

impl ContractArtifact {
    pub fn function_as_compiled_program(&self, function_name: &str) -> Option<CompiledProgram> {
        self.functions.iter().find(|f| f.name == function_name).map(|f| {
            f.clone().into_compiled_program(self.noir_version.clone(), self.file_map.clone())
        })
    }
}

/// Each function in the contract will be compiled as a separate noir program.
///
/// A contract function unlike a regular Noir program however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFunctionArtifact {
    pub name: String,

    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which the [`ContractFunction`]
    /// was compiled.
    #[serde(default)] // For backwards compatibility (it was missing).
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: u64,

    pub is_unconstrained: bool,

    pub custom_attributes: Vec<String>,

    pub abi: Abi,

    #[serde(
        serialize_with = "Program::serialize_program_base64",
        deserialize_with = "Program::deserialize_program_base64"
    )]
    pub bytecode: Program<FieldElement>,

    #[serde(
        serialize_with = "ProgramDebugInfo::serialize_compressed_base64_json",
        deserialize_with = "ProgramDebugInfo::deserialize_compressed_base64_json"
    )]
    pub debug_symbols: ProgramDebugInfo,
    #[serde(default)] // For backwards compatibility (it was missing).
    pub names: Vec<String>,
    pub brillig_names: Vec<String>,
}

impl ContractFunctionArtifact {
    pub fn into_compiled_program(
        self,
        noir_version: String,
        file_map: BTreeMap<FileId, DebugFile>,
    ) -> CompiledProgram {
        CompiledProgram {
            noir_version,
            hash: self.hash,
            program: self.bytecode,
            abi: self.abi,
            debug: self.debug_symbols.debug_infos,
            file_map,
            warnings: Vec::new(),
            names: self.names,
            brillig_names: self.brillig_names,
        }
    }
}

impl From<ContractFunction> for ContractFunctionArtifact {
    fn from(func: ContractFunction) -> Self {
        ContractFunctionArtifact {
            name: func.name,
            hash: func.hash,
            is_unconstrained: func.is_unconstrained,
            custom_attributes: func.custom_attributes,
            abi: func.abi,
            bytecode: func.bytecode,
            names: func.names,
            brillig_names: func.brillig_names,
            debug_symbols: ProgramDebugInfo { debug_infos: func.debug },
        }
    }
}
