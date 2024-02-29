use std::collections::BTreeMap;

use acvm::acir::circuit::Circuit;
use fm::FileId;
use noirc_abi::Abi;
use noirc_driver::CompiledProgram;
use noirc_driver::DebugFile;
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramArtifact {
    pub noir_version: String,

    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`ProgramArtifact`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

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

    /// Map of file Id to the source code so locations in debug info can be mapped to source code they point to.
    pub file_map: BTreeMap<FileId, DebugFile>,
}

impl From<CompiledProgram> for ProgramArtifact {
    fn from(program: CompiledProgram) -> Self {
        ProgramArtifact {
            hash: program.hash,
            abi: program.abi,
            noir_version: program.noir_version,
            bytecode: program.circuit,
            debug_symbols: program.debug,
            file_map: program.file_map,
        }
    }
}

impl From<ProgramArtifact> for CompiledProgram {
    fn from(program: ProgramArtifact) -> Self {
        CompiledProgram {
            hash: program.hash,
            abi: program.abi,
            noir_version: program.noir_version,
            circuit: program.bytecode,
            debug: program.debug_symbols,
            file_map: program.file_map,
            warnings: vec![],
        }
    }
}
