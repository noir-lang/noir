use std::collections::BTreeMap;

use acvm::FieldElement;
use acvm::acir::circuit::ExpressionWidth;
use acvm::acir::circuit::Program;
use fm::FileId;
use noirc_abi::Abi;
use serde::{Deserialize, Serialize};

use crate::debug::DebugFile;
use crate::debug::DebugInfo;
use crate::debug::ProgramDebugInfo;
use crate::ssa::SsaReport;

use super::{deserialize_hash, serialize_hash};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProgramArtifact {
    pub noir_version: String,

    /// Hash of the monomorphized program from which this [`ProgramArtifact`] was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: u64,

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

    /// Map of file Id to the source code so locations in debug info can be mapped to source code they point to.
    pub file_map: BTreeMap<FileId, DebugFile>,
    /// Maximum width of the expressions which will be constrained
    pub expression_width: ExpressionWidth,
}

impl From<CompiledProgram> for ProgramArtifact {
    fn from(compiled_program: CompiledProgram) -> Self {
        ProgramArtifact {
            hash: compiled_program.hash,
            abi: compiled_program.abi,
            noir_version: compiled_program.noir_version,
            bytecode: compiled_program.program,
            debug_symbols: ProgramDebugInfo { debug_infos: compiled_program.debug },
            file_map: compiled_program.file_map,
            expression_width: compiled_program.expression_width,
        }
    }
}

impl From<ProgramArtifact> for CompiledProgram {
    fn from(program: ProgramArtifact) -> Self {
        CompiledProgram {
            hash: program.hash,
            abi: program.abi,
            noir_version: program.noir_version,
            program: program.bytecode,
            debug: program.debug_symbols.debug_infos,
            file_map: program.file_map,
            warnings: vec![],
            expression_width: program.expression_width,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct CompiledProgram {
    pub noir_version: String,
    /// Hash of the Program (noirc_frontend::monomorphization::ast::Program) from which this [`CompiledProgram`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    #[serde(
        serialize_with = "Program::serialize_program_base64",
        deserialize_with = "Program::deserialize_program_base64"
    )]
    pub program: Program<FieldElement>,
    pub abi: Abi,
    pub debug: Vec<DebugInfo>,
    pub file_map: BTreeMap<FileId, DebugFile>,
    pub warnings: Vec<SsaReport>,
    /// Maximum width of the expressions which will be constrained
    pub expression_width: ExpressionWidth,
}
