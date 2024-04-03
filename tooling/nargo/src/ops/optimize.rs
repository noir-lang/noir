use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

pub fn optimize_program(
    mut compiled_program: CompiledProgram,
) -> CompiledProgram {
    let functions = std::mem::take(&mut compiled_program.program.functions);

    let optimized_functions = functions.into_iter().enumerate().map(|(i, function)| {
        let (optimized_circuit, location_map) = acvm::compiler::optimize(
            function
        );
        compiled_program.debug[i].update_acir(location_map);
        optimized_circuit
    }).collect::<Vec<_>>();

    compiled_program.program.functions = optimized_functions;
    compiled_program
}

pub fn optimize_contract(
    contract: CompiledContract,
) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let functions = std::mem::take(&mut func.bytecode.functions);

        let optimized_functions = functions.into_iter().enumerate().map(|(i, function)| {
            let (optimized_circuit, location_map) = acvm::compiler::optimize(
                function
            );
            func.debug[i].update_acir(location_map);
            optimized_circuit
        }).collect::<Vec<_>>();

        func.bytecode.functions = optimized_functions;
        func
    });
    
    CompiledContract { functions, ..contract }
}

