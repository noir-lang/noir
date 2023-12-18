use acvm::Language;
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

pub fn optimize_program(mut program: CompiledProgram, np_language: Language) -> CompiledProgram {
    let (optimized_circuit, location_map) = acvm::compiler::compile(program.circuit, np_language);

    program.circuit = optimized_circuit;
    program.debug.update_acir(location_map);
    program
}

pub fn optimize_contract(contract: CompiledContract, np_language: Language) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) =
            acvm::compiler::compile(func.bytecode, np_language);
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        func
    });

    CompiledContract { functions, ..contract }
}
