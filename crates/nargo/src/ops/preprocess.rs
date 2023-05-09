use acvm::ProofSystemCompiler;
use noirc_driver::{CompiledContract, CompiledProgram};

use crate::artifacts::{
    contract::{PreprocessedContract, PreprocessedContractFunction},
    program::PreprocessedProgram,
};

// TODO: pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub fn preprocess_program<B: ProofSystemCompiler>(
    backend: &B,
    compiled_program: CompiledProgram,
) -> Result<PreprocessedProgram, B::Error> {
    // TODO: currently `compiled_program`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = compiled_program.circuit;
    let (proving_key, verification_key) = backend.preprocess(&optimized_bytecode)?;

    Ok(PreprocessedProgram {
        backend: String::from(BACKEND_IDENTIFIER),
        abi: compiled_program.abi,
        bytecode: optimized_bytecode,
        proving_key,
        verification_key,
    })
}

pub fn preprocess_contract<B: ProofSystemCompiler>(
    backend: &B,
    compiled_contract: CompiledContract,
) -> Result<PreprocessedContract, B::Error> {
    let mut preprocessed_contract_functions = vec![];
    for func in compiled_contract.functions.into_iter() {
        // TODO: currently `func`'s bytecode is already optimized for the backend.
        // In future we'll need to apply those optimizations here.
        let optimized_bytecode = func.bytecode;
        let (proving_key, verification_key) = backend.preprocess(&optimized_bytecode)?;

        let preprocessed = PreprocessedContractFunction {
            name: func.name,
            function_type: func.function_type,
            abi: func.abi,

            bytecode: optimized_bytecode,
            proving_key,
            verification_key,
        };

        preprocessed_contract_functions.push(preprocessed);
    }

    Ok(PreprocessedContract {
        name: compiled_contract.name,
        backend: String::from(BACKEND_IDENTIFIER),
        functions: preprocessed_contract_functions,
    })
}
