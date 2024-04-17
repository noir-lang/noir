use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

/// TODO(https://github.com/noir-lang/noir/issues/4428): Need to update how these passes are run to account for
/// multiple ACIR functions

pub fn optimize_program(mut compiled_program: CompiledProgram) -> CompiledProgram {
    let (optimized_circuit, location_map) =
        acvm::compiler::optimize(std::mem::take(&mut compiled_program.program.functions[0]));
    compiled_program.program.functions[0] = optimized_circuit;
    compiled_program.debug.update_acir(location_map);
    compiled_program
}

pub fn optimize_contract(contract: CompiledContract) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) =
            acvm::compiler::optimize(std::mem::take(&mut func.bytecode.functions[0]));
        func.bytecode.functions[0] = optimized_bytecode;
        func.debug.update_acir(location_map);
        func
    });

    CompiledContract { functions, ..contract }
}
