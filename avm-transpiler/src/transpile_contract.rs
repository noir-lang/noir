use std::io::Read;

use acvm::FieldElement;
use base64::Engine;
use log::info;
use serde::{Deserialize, Serialize};

use acvm::acir::circuit::Program;
use noirc_errors::debug_info::ProgramDebugInfo;

use crate::transpile::{brillig_to_avm, map_brillig_pcs_to_avm_pcs, patch_debug_info_pcs};
use crate::utils::extract_brillig_from_acir_program;

/// Representation of a contract with some transpiled functions
#[derive(Debug, Serialize, Deserialize)]
pub struct TranspiledContractArtifact {
    pub transpiled: bool,
    pub noir_version: String,
    pub name: String,
    // Functions can be ACIR or AVM
    pub functions: Vec<AvmOrAcirContractFunctionArtifact>,
    pub outputs: serde_json::Value,
    pub file_map: serde_json::Value,
}

/// A regular contract with ACIR+Brillig functions
/// but with fields irrelevant to transpilation
/// represented as catch-all serde Values
#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledAcirContractArtifact {
    pub noir_version: String,
    pub name: String,
    pub functions: Vec<AcirContractFunctionArtifact>,
    pub outputs: serde_json::Value,
    pub file_map: serde_json::Value,
}

/// Representation of a contract function
/// with AVM bytecode as a base64 string
#[derive(Debug, Serialize, Deserialize)]
pub struct AvmContractFunctionArtifact {
    pub name: String,
    pub is_unconstrained: bool,
    pub custom_attributes: Vec<String>,
    pub abi: serde_json::Value,
    pub bytecode: String, // base64
    #[serde(
        serialize_with = "ProgramDebugInfo::serialize_compressed_base64_json",
        deserialize_with = "ProgramDebugInfo::deserialize_compressed_base64_json"
    )]
    pub debug_symbols: ProgramDebugInfo,
}

/// Representation of an ACIR contract function but with
/// catch-all serde Values for fields irrelevant to transpilation
#[derive(Debug, Serialize, Deserialize)]
pub struct AcirContractFunctionArtifact {
    pub name: String,
    pub is_unconstrained: bool,
    pub custom_attributes: Vec<String>,
    pub abi: serde_json::Value,
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
}

/// An enum that allows the TranspiledContract struct to contain
/// functions with either ACIR or AVM bytecode
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)] // omit Acir/Avm tag for these objects in json
pub enum AvmOrAcirContractFunctionArtifact {
    Acir(AcirContractFunctionArtifact),
    Avm(AvmContractFunctionArtifact),
}

/// Transpilation is performed when a TranspiledContract
/// is constructed from a CompiledAcirContract
impl From<CompiledAcirContractArtifact> for TranspiledContractArtifact {
    fn from(contract: CompiledAcirContractArtifact) -> Self {
        let mut functions: Vec<AvmOrAcirContractFunctionArtifact> = Vec::new();

        for function in contract.functions {
            // TODO(4269): once functions are tagged for transpilation to AVM, check tag
            if function.custom_attributes.contains(&"aztec(public)".to_string()) {
                info!("Transpiling AVM function {} on contract {}", function.name, contract.name);
                // Extract Brillig Opcodes from acir
                let acir_program = function.bytecode;
                let brillig_bytecode = extract_brillig_from_acir_program(&acir_program);

                // Map Brillig pcs to AVM pcs (index is Brillig PC, value is AVM PC)
                let brillig_pcs_to_avm_pcs = map_brillig_pcs_to_avm_pcs(brillig_bytecode);

                // Transpile to AVM
                let avm_bytecode = brillig_to_avm(brillig_bytecode, &brillig_pcs_to_avm_pcs);

                // Gzip AVM bytecode. This has to be removed once we need to do bytecode verification.
                let mut compressed_avm_bytecode = Vec::new();
                let mut encoder =
                    flate2::read::GzEncoder::new(&avm_bytecode[..], flate2::Compression::best());
                let _ = encoder.read_to_end(&mut compressed_avm_bytecode);

                log::info!(
                    "{}::{}: compressed {} to {} bytes ({}% reduction)",
                    contract.name,
                    function.name,
                    avm_bytecode.len(),
                    compressed_avm_bytecode.len(),
                    100 - (compressed_avm_bytecode.len() * 100 / avm_bytecode.len())
                );

                // Patch the debug infos with updated PCs
                let debug_infos = patch_debug_info_pcs(
                    function.debug_symbols.debug_infos,
                    &brillig_pcs_to_avm_pcs,
                );

                // Push modified function entry to ABI
                functions.push(AvmOrAcirContractFunctionArtifact::Avm(
                    AvmContractFunctionArtifact {
                        name: function.name,
                        is_unconstrained: function.is_unconstrained,
                        custom_attributes: function.custom_attributes,
                        abi: function.abi,
                        bytecode: base64::prelude::BASE64_STANDARD.encode(compressed_avm_bytecode),
                        debug_symbols: ProgramDebugInfo { debug_infos },
                    },
                ));
            } else {
                // This function is not flagged for transpilation. Push original entry.
                functions.push(AvmOrAcirContractFunctionArtifact::Acir(function));
            }
        }
        TranspiledContractArtifact {
            transpiled: true,
            noir_version: contract.noir_version,
            name: contract.name,
            functions, // some acir, some transpiled avm functions
            outputs: contract.outputs,
            file_map: contract.file_map,
        }
    }
}
