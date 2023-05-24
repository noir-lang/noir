use acvm::{
    acir::circuit::Circuit, compiler::optimizers::simplify::CircuitSimplifier, ProofSystemCompiler,
};
use noirc_driver::{CompiledProgram, ContractFunction};

use crate::{
    artifacts::{contract::PreprocessedContractFunction, program::PreprocessedProgram},
    NargoError,
};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

pub fn preprocess_program<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    compiled_program: CompiledProgram,
) -> Result<PreprocessedProgram, B::Error> {
    let optimized_bytecode = optimize_circuit(backend, compiled_program.circuit).unwrap();
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
    let optimized_bytecode = optimize_circuit(backend, func.bytecode).unwrap();
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

fn optimize_circuit(
    backend: &impl ProofSystemCompiler,
    circuit: Circuit,
) -> Result<Circuit, NargoError> {
    let simplifier = CircuitSimplifier::new(circuit.current_witness_index);
    let optimized_circuit = acvm::compiler::compile(
        circuit,
        backend.np_language(),
        |opcode| backend.supports_opcode(opcode),
        &simplifier,
    )
    .unwrap();

    Ok(optimized_circuit)
}
