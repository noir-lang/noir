use acvm::ProofSystemCompiler;

use acvm::acir::circuit::Circuit;
use base64::Engine;
use noirc_abi::Abi;
use noirc_driver::FunctionType;
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::optimize::OptimizedFunction;

/// `PreprocessedContract` represents a Noir contract which has been preprocessed by a particular backend proving system.
///
/// This differs from a generic Noir contract artifact in that:
/// - The ACIR bytecode has had an optimization pass applied to tailor it for the backend.
/// - Proving and verification keys have been pregenerated based on this ACIR.
#[derive(Serialize, Deserialize)]
pub struct PreprocessedProgram {
    /// The name of the contract.
    pub name: Option<String>,
    /// The identifier of the proving backend which this contract has been compiled for.
    pub backend: String,
    /// Each of the contract's functions are compiled into a separate program stored in this `Vec`.
    pub functions: Vec<PreprocessedFunction>,
}

/// Each function in the contract will be compiled as a separate noir program.
///
/// A contract function unlike a regular Noir program however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreprocessedFunction {
    pub name: String,

    pub function_type: FunctionType,

    pub is_internal: bool,

    pub abi: Abi,

    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub bytecode: Circuit,

    pub proving_key: Option<Vec<u8>>,
    pub verification_key: Option<Vec<u8>>,

    #[serde(skip)]
    pub debug: DebugInfo,
}

// TODO: move these down into ACVM.
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();
    let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
    s.serialize_str(&encoded_b64)
}

fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
    let circuit_bytes = base64::engine::general_purpose::STANDARD.decode(bytecode_b64).unwrap();
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}

pub fn preprocess_function<B: ProofSystemCompiler>(
    backend: &B,
    include_keys: bool,
    common_reference_string: &[u8],
    func: OptimizedFunction,
) -> Result<PreprocessedFunction, B::Error> {
    // TODO: currently `compiled_program`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.

    let optimized_bytecode = func.bytecode;

    let (proving_key, verification_key) = if include_keys {
        let (proving_key, verification_key) =
            backend.preprocess(common_reference_string, &optimized_bytecode)?;
        (Some(proving_key), Some(verification_key))
    } else {
        (None, None)
    };

    Ok(PreprocessedFunction {
        name: func.name,
        function_type: func.function_type,
        is_internal: func.is_internal,
        abi: func.abi,

        bytecode: optimized_bytecode,
        proving_key,
        verification_key,

        debug: func.debug,
    })
}
