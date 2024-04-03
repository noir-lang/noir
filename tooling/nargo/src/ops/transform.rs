use acvm::{acir::circuit::ExpressionWidth, compiler::AcirTransformationMap};
use iter_extended::vecmap;
use noirc_driver::{CompiledContract, CompiledProgram};

/// TODO(https://github.com/noir-lang/noir/issues/4428): Need to update how these passes are run to account for
/// multiple ACIR functions
// TODO: old one delete this, just kept for testing
// pub fn transform_program(
//     mut compiled_program: CompiledProgram,
//     expression_width: ExpressionWidth,
// ) -> CompiledProgram {
//     let (optimized_circuit, location_map) = acvm::compiler::compile(
//         std::mem::take(&mut compiled_program.program.functions[0]),
//         expression_width,
//     );

//     compiled_program.program.functions[0] = optimized_circuit;
//     compiled_program.debug[0].update_acir(location_map);
//     compiled_program
// }

pub fn transform_program(
    mut compiled_program: CompiledProgram,
    expression_width: ExpressionWidth,
) -> CompiledProgram {
    let functions = std::mem::take(&mut compiled_program.program.functions);
    dbg!(functions.clone());
    dbg!(compiled_program.program.functions.len());
    let optimized_functions = functions.into_iter().enumerate().map(|(i, function)| {
        if i == 0 {
            dbg!(function.clone());
        }
        let (optimized_circuit, location_map) = acvm::compiler::compile(
            function,
            expression_width
        );
        if i == 0 {
            dbg!(optimized_circuit.clone());
        }
        compiled_program.debug[i].update_acir(location_map);
        optimized_circuit
    }).collect::<Vec<_>>();
    dbg!(optimized_functions.clone());
    compiled_program.program.functions = optimized_functions;
    compiled_program
}

pub fn transform_contract(
    contract: CompiledContract,
    expression_width: ExpressionWidth,
) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        let functions = std::mem::take(&mut func.bytecode.functions);

        let optimized_functions = functions.into_iter().enumerate().map(|(i, function)| {
            let (optimized_circuit, location_map) = acvm::compiler::compile(
                function,
                expression_width
            );
            func.debug[i].update_acir(location_map);
            optimized_circuit
        }).collect::<Vec<_>>();

        func.bytecode.functions = optimized_functions;
        func
    });
    
    CompiledContract { functions, ..contract }
}

// pub fn transform_contract(
//     contract: CompiledContract,
//     expression_width: ExpressionWidth,
// ) -> CompiledContract {
//     let functions = vecmap(contract.functions, |mut func| {
//         let (optimized_bytecode, location_map) = acvm::compiler::compile(
//             std::mem::take(&mut func.bytecode.functions[0]),
//             expression_width,
//         );
//         func.bytecode.functions[0] = optimized_bytecode;
//         func.debug.update_acir(location_map);
//         func
//     });

//     CompiledContract { functions, ..contract }
// }
