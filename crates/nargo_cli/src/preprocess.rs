use acvm::ProofSystemCompiler;
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

// TODO: migrate to `nargo_cli`

use crate::artifacts::{
    contract::{PreprocessedContract, PreprocessedContractFunction},
    program::PreprocessedProgram,
};

// TODO: pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub(crate) fn preprocess_program(compiled_program: CompiledProgram) -> PreprocessedProgram {
    let backend = crate::backends::ConcreteBackend;

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

pub(crate) fn preprocess_contract(compiled_contract: CompiledContract) -> PreprocessedContract {
    let backend = crate::backends::ConcreteBackend;

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
