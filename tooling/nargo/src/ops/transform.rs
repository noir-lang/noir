use acvm::ExpressionWidth;
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

pub fn transform_program(
    mut program: CompiledProgram,
    expression_width: ExpressionWidth,
) -> CompiledProgram {
    let (optimized_circuit, location_map) =
        acvm::compiler::compile(program.circuit, expression_width);

    program.circuit = optimized_circuit;
    program.debug.update_acir(location_map);
    program
}

pub fn transform_contract(
    contract: CompiledContract,
    expression_width: ExpressionWidth,
) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) =
            acvm::compiler::compile(func.bytecode, expression_width);
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        func
    });

    CompiledContract { functions, ..contract }
}
