use acvm::ProofSystemCompiler;
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

use crate::artifacts::{
    contract::{PreprocessedContract, PreprocessedContractFunction},
    program::PreprocessedProgram,
};

// TODO: pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub fn preprocess_program(
    backend: &impl ProofSystemCompiler,
    compiled_program: CompiledProgram,
) -> PreprocessedProgram {
    // TODO: currently `compiled_program`'s bytecode is already optimized for the backend.
    // In future we'll need to apply those optimizations here.
    let optimized_bytecode = compiled_program.circuit;
    let (proving_key, verification_key) = backend.preprocess(&optimized_bytecode);

    PreprocessedProgram {
        backend: String::from(BACKEND_IDENTIFIER),
        abi: compiled_program.abi,
        bytecode: optimized_bytecode,
        proving_key,
        verification_key,
    }
}

pub fn preprocess_contract(
    backend: &impl ProofSystemCompiler,
    compiled_contract: CompiledContract,
) -> PreprocessedContract {
    let preprocessed_contract_functions = vecmap(compiled_contract.functions, |func| {
        // TODO: currently `func`'s bytecode is already optimized for the backend.
        // In future we'll need to apply those optimizations here.
        let optimized_bytecode = func.bytecode;
        let (proving_key, verification_key) = backend.preprocess(&optimized_bytecode);

        PreprocessedContractFunction {
            name: func.name,
            function_type: func.function_type,
            abi: func.abi,

            bytecode: optimized_bytecode,
            proving_key,
            verification_key,
        }
    });

    PreprocessedContract {
        name: compiled_contract.name,
        backend: String::from(BACKEND_IDENTIFIER),
        functions: preprocessed_contract_functions,
    }
}
