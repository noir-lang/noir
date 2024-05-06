use acvm::acir::circuit::Program;
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};
use noirc_errors::debug_info::DebugInfo;

pub fn optimize_program(mut compiled_program: CompiledProgram) -> CompiledProgram {
    compiled_program.program =
        optimize_program_internal(compiled_program.program, &mut compiled_program.debug);
    compiled_program
}

pub fn optimize_contract(contract: CompiledContract) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        func.bytecode = optimize_program_internal(func.bytecode, &mut func.debug);
        func
    });

    CompiledContract { functions, ..contract }
}

fn optimize_program_internal(mut program: Program, debug: &mut [DebugInfo]) -> Program {
    let functions = std::mem::take(&mut program.functions);

    let optimized_functions = functions
        .into_iter()
        .enumerate()
        .map(|(i, function)| {
            let (optimized_circuit, location_map) = acvm::compiler::optimize(function);
            debug[i].update_acir(location_map);
            optimized_circuit
        })
        .collect::<Vec<_>>();

    program.functions = optimized_functions;
    program
}
