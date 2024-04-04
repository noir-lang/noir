use base64::Engine;
use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};

use acvm::acir::circuit::Program;

use crate::transpile::brillig_to_avm;
use crate::utils::extract_brillig_from_acir;

/// Representation of a contract with some transpiled functions
#[derive(Debug, Serialize, Deserialize)]
pub struct TranspiledContract {
    pub transpiled: bool,
    pub noir_version: String,
    pub name: String,
    // Functions can be ACIR or AVM
    pub functions: Vec<AvmOrAcirContractFunction>,
    pub outputs: serde_json::Value,
    pub file_map: serde_json::Value,
    //pub warnings: serde_json::Value,
}

/// A regular contract with ACIR+Brillig functions
/// but with fields irrelevant to transpilation
/// represented as catch-all serde Values
#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledAcirContract {
    pub noir_version: String,
    pub name: String,
    pub functions: Vec<AcirContractFunction>,
    pub outputs: serde_json::Value,
    pub file_map: serde_json::Value,
    //pub warnings: serde_json::Value,
}

/// Representation of a contract function
/// with AVM bytecode as a base64 string
#[derive(Debug, Serialize, Deserialize)]
pub struct AvmContractFunction {
    pub name: String,
    pub is_unconstrained: bool,
    pub custom_attributes: Vec<String>,
    pub abi: serde_json::Value,
    pub bytecode: String, // base64
    pub debug_symbols: serde_json::Value,
}

/// Representation of an ACIR contract function but with
/// catch-all serde Values for fields irrelevant to transpilation
#[derive(Debug, Serialize, Deserialize)]
pub struct AcirContractFunction {
    pub name: String,
    pub is_unconstrained: bool,
    pub custom_attributes: Vec<String>,
    pub abi: serde_json::Value,
    #[serde(
        serialize_with = "Program::serialize_program_base64",
        deserialize_with = "Program::deserialize_program_base64"
    )]
    pub bytecode: Program,
    pub debug_symbols: serde_json::Value,
}

/// An enum that allows the TranspiledContract struct to contain
/// functions with either ACIR or AVM bytecode
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)] // omit Acir/Avm tag for these objects in json
pub enum AvmOrAcirContractFunction {
    Acir(AcirContractFunction),
    Avm(AvmContractFunction),
}

/// Transpilation is performed when a TranspiledContract
/// is constructed from a CompiledAcirContract
impl From<CompiledAcirContract> for TranspiledContract {
    fn from(contract: CompiledAcirContract) -> Self {
        let mut functions = Vec::new();

        for function in contract.functions {
            // TODO(4269): once functions are tagged for transpilation to AVM, check tag
            if function
                .custom_attributes
                .contains(&"aztec(public-vm)".to_string())
            {
                info!(
                    "Transpiling AVM function {} on contract {}",
                    function.name, contract.name
                );
                // Extract Brillig Opcodes from acir
                let acir_program = function.bytecode;
                let brillig = extract_brillig_from_acir(&acir_program.functions[0].opcodes);

                // Transpile to AVM
                let avm_bytecode = brillig_to_avm(brillig);

                // Push modified function entry to ABI
                functions.push(AvmOrAcirContractFunction::Avm(AvmContractFunction {
                    name: function.name,
                    is_unconstrained: function.is_unconstrained,
                    custom_attributes: function.custom_attributes,
                    abi: function.abi,
                    bytecode: base64::prelude::BASE64_STANDARD.encode(avm_bytecode),
                    debug_symbols: function.debug_symbols,
                }));
            } else {
                // This function is not flagged for transpilation. Push original entry.
                functions.push(AvmOrAcirContractFunction::Acir(function));
            }
        }
        TranspiledContract {
            transpiled: true,
            noir_version: contract.noir_version,
            name: contract.name,
            functions, // some acir, some transpiled avm functions
            outputs: contract.outputs,
            file_map: contract.file_map,
            //warnings: contract.warnings,
        }
    }
}
