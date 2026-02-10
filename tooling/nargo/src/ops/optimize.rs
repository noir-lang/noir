use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::circuit::{Program, brillig::BrilligFunctionId},
};
use iter_extended::vecmap;
use noirc_artifacts::{contract::CompiledContract, debug::DebugInfo, program::CompiledProgram};

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

fn optimize_program_internal(
    mut program: Program<FieldElement>,
    debug: &mut [DebugInfo],
) -> Program<FieldElement> {
    let functions = std::mem::take(&mut program.functions);

    let brillig_side_effects = brillig_side_effects(&program);

    let optimized_functions = functions
        .into_iter()
        .enumerate()
        .map(|(i, function)| {
            let (optimized_circuit, location_map) =
                acvm::compiler::optimize(function, &brillig_side_effects);
            debug[i].update_acir(location_map);
            optimized_circuit
        })
        .collect::<Vec<_>>();

    program.functions = optimized_functions;
    program
}

/// Collect information whether Brillig functions might have side effects.
pub(super) fn brillig_side_effects(
    program: &Program<FieldElement>,
) -> BTreeMap<BrilligFunctionId, bool> {
    program
        .unconstrained_functions
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let id = BrilligFunctionId(idx as u32);
            let has_side_effect = f
                .bytecode
                .iter()
                .any(|opcode| matches!(opcode, brillig::Opcode::ForeignCall { .. }));
            (id, has_side_effect)
        })
        .collect()
}
