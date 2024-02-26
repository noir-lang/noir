use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

pub fn optimize_program(mut program: CompiledProgram) -> CompiledProgram {
    let (optimized_circuit, location_map) = acvm::compiler::optimize(program.circuit);
    program.circuit = optimized_circuit;
    program.debug.update_acir(location_map);
    program
}

pub fn optimize_contract(contract: CompiledContract) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) = acvm::compiler::optimize(func.bytecode);
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        func
    });

    CompiledContract { functions, ..contract }
}
