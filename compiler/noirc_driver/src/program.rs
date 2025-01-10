use std::collections::BTreeMap;

use acvm::{acir::circuit::Program, FieldElement};
use fm::FileId;

use noirc_errors::debug_info::DebugInfo;
use noirc_evaluator::errors::SsaReport;
use serde::{Deserialize, Serialize};

use super::debug::DebugFile;

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct CompiledProgram {
    pub noir_version: String,
    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`CompiledProgram`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    #[serde(
        serialize_with = "Program::serialize_program_base64",
        deserialize_with = "Program::deserialize_program_base64"
    )]
    pub program: Program<FieldElement>,
    pub abi: noirc_abi::Abi,
    pub debug: Vec<DebugInfo>,
    pub file_map: BTreeMap<FileId, DebugFile>,
    pub warnings: Vec<SsaReport>,
    /// Names of the functions in the program. These are used for more informative debugging and benchmarking.
    pub names: Vec<String>,
    /// Names of the unconstrained functions in the program.
    pub brillig_names: Vec<String>,
}
