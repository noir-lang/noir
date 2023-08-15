use acvm::ProofSystemCompiler;
use noirc_driver::{CompiledProgram, ContractFunction};
use noirc_errors::debug_info::DebugInfo;

use crate::artifacts::{contract::PreprocessedContractFunction, program::PreprocessedProgram};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub fn preprocess_program<B: ProofSystemCompiler>(
    backend: &B,
    include_keys: bool,
    common_reference_string: &[u8],
    compiled_program: CompiledProgram,
) -> Result<(PreprocessedProgram, DebugInfo), B::Error> {
    // TODO: currently `compiled_program`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = compiled_program.circuit;

    let (proving_key, verification_key) = if include_keys {
        let (proving_key, verification_key) =
            backend.preprocess(common_reference_string, &optimized_bytecode)?;
        (Some(proving_key), Some(verification_key))
    } else {
        (None, None)
    };

    Ok((
        PreprocessedProgram {
            backend: String::from(BACKEND_IDENTIFIER),
            abi: compiled_program.abi,
            bytecode: optimized_bytecode,
            proving_key,
            verification_key,
        },
        compiled_program.debug,
    ))
}

pub fn preprocess_contract_function<B: ProofSystemCompiler>(
    backend: &B,
    include_keys: bool,
    common_reference_string: &[u8],
    func: ContractFunction,
) -> Result<(PreprocessedContractFunction, DebugInfo), B::Error> {
    // TODO: currently `func`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = func.bytecode;
    let (proving_key, verification_key) = if include_keys {
        let (proving_key, verification_key) =
            backend.preprocess(common_reference_string, &optimized_bytecode)?;
        (Some(proving_key), Some(verification_key))
    } else {
        (None, None)
    };

    Ok((
        PreprocessedContractFunction {
            name: func.name,
            function_type: func.function_type,
            is_internal: func.is_internal,
            abi: func.abi,

            bytecode: optimized_bytecode,
            proving_key,
            verification_key,
        },
        func.debug,
    ))
}
