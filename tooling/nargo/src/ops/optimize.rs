use acvm::Language;
use iter_extended::try_vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

use crate::NargoError;

pub fn optimize_program(
    mut program: CompiledProgram,
    np_language: Language,
) -> Result<CompiledProgram, NargoError> {
    let (optimized_circuit, location_map) = acvm::compiler::compile(program.circuit, np_language);

    program.circuit = optimized_circuit;
    program.debug.update_acir(location_map);
    Ok(program)
}

pub fn optimize_contract(
    contract: CompiledContract,
    np_language: Language,
) -> Result<CompiledContract, NargoError> {
    let functions = try_vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) =
            acvm::compiler::compile(func.bytecode, np_language);
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        Ok::<_, NargoError>(func)
    })?;

    Ok(CompiledContract { functions, ..contract })
}
