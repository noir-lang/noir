use acvm::ProofSystemCompiler;
use noirc_driver::{CompiledProgram, ContractFunction};

use crate::artifacts::{contract::PreprocessedContractFunction, program::PreprocessedProgram};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub fn preprocess_program<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    compiled_program: CompiledProgram,
) -> Result<PreprocessedProgram, B::Error> {
    // TODO: currently `compiled_program`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = compiled_program.circuit;
    let (proving_key, verification_key) =
        backend.preprocess(common_reference_string, &optimized_bytecode)?;

    Ok(PreprocessedProgram {
        backend: String::from(BACKEND_IDENTIFIER),
        abi: compiled_program.abi,
        bytecode: optimized_bytecode,
        proving_key,
        verification_key,
    })
}

pub fn preprocess_contract_function<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    func: ContractFunction,
) -> Result<PreprocessedContractFunction, B::Error> {
    // TODO: currently `func`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = func.bytecode;
    let (proving_key, verification_key) =
        backend.preprocess(common_reference_string, &optimized_bytecode)?;

    Ok(PreprocessedContractFunction {
        name: func.name,
        function_type: func.function_type,
        abi: func.abi,

        bytecode: optimized_bytecode,
        proving_key,
        verification_key,
    })
}
