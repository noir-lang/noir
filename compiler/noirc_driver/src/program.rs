use std::collections::BTreeMap;

use acvm::acir::circuit::Circuit;
use fm::FileId;

use noirc_errors::debug_info::DebugInfo;
use noirc_evaluator::errors::SsaReport;
use serde::{Deserialize, Serialize};

use super::debug::DebugFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    pub noir_version: String,
    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`CompiledProgram`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    #[serde(
        serialize_with = "Circuit::serialize_circuit_base64",
        deserialize_with = "Circuit::deserialize_circuit_base64"
    )]
    pub circuit: Circuit,
    pub abi: noirc_abi::Abi,
    pub debug: DebugInfo,
    pub file_map: BTreeMap<FileId, DebugFile>,
    pub warnings: Vec<SsaReport>,
}
