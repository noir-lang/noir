use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::circuit::{Program, brillig::BrilligFunctionId},
};
use iter_extended::vecmap;
use noirc_artifacts::{contract::CompiledContract, debug::DebugInfo, program::CompiledProgram};

/// Re-run the backend-independent and backend-specific optimization passes over an
/// already-compiled program.
///
/// ACIR generation already optimizes each circuit, but it does so without knowing
/// which Brillig functions have side effects (that information is only assembled into
/// the full [`Program`] afterwards). This pass exists to take advantage of that extra
/// information. If it adds nothing over the conservative default assumed during
/// codegen, the program is returned unchanged rather than re-optimized.
pub fn optimize_program(mut compiled_program: CompiledProgram) -> CompiledProgram {
    compiled_program.program =
        optimize_program_internal(compiled_program.program, &mut compiled_program.debug);
    compiled_program
}

/// Re-run optimization over each function of an already-compiled contract.
///
/// See [`optimize_program`]: each function is only actually re-optimized when there is
/// more to do than the conservative default already applied during ACIR generation.
pub fn optimize_contract(contract: CompiledContract) -> CompiledContract {
    let functions = vecmap(contract.functions, |mut func| {
        func.bytecode = optimize_program_internal(func.bytecode, &mut func.debug);
        func
    });

    CompiledContract { functions, ..contract }
}

/// Optimize each function of a program, given the assembled program so we can derive
/// Brillig side-effect information.
///
/// ACIR generation already ran this same optimization pass over each circuit, but with
/// no Brillig side-effect information. The `RangeOptimizer` treats missing information
/// as "this call may have side effects" (`unwrap_or(true)`), so the only thing this
/// post-codegen pass can additionally do is remove range constraints guarding a call we
/// now know to be side-effect free. If no such call exists, re-optimizing reproduces the
/// identical circuit, so we skip the expensive backend transform and return the program
/// untouched.
fn optimize_program_internal(
    mut program: Program<FieldElement>,
    debug: &mut [DebugInfo],
) -> Program<FieldElement> {
    let brillig_side_effects = brillig_side_effects(&program);

    if !has_side_effect_free_brillig(&brillig_side_effects) {
        return program;
    }

    let functions = std::mem::take(&mut program.functions);

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

/// Whether any Brillig function is known to be free of side effects.
///
/// This is the only new information available to the post-codegen optimization pass
/// compared to the side-effect-unaware pass run during ACIR generation, so it gates
/// whether re-optimizing the program can change anything.
fn has_side_effect_free_brillig(brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>) -> bool {
    brillig_side_effects.values().any(|has_side_effect| !has_side_effect)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acvm::{
        FieldElement,
        acir::circuit::{Circuit, Program, brillig::BrilligFunctionId},
    };
    use noirc_artifacts::debug::DebugInfo;

    use super::{has_side_effect_free_brillig, optimize_program_internal};

    /// A circuit whose linear combination is wider than the CSAT width (4), so the
    /// backend transform would split it into intermediate variables if it ran.
    fn wide_circuit() -> Circuit<FieldElement> {
        let src = "
        private parameters: [w0, w1, w2, w3, w4, w5, w6, w7]
        public parameters: []
        return values: [w8]
        ASSERT w8 = w0 + w1 + w2 + w3 + w4 + w5 + w6 + w7
        ";
        Circuit::from_str(src).unwrap()
    }

    #[test]
    fn detects_side_effect_free_brillig() {
        assert!(!has_side_effect_free_brillig(&BTreeMap::new()));
        assert!(!has_side_effect_free_brillig(&BTreeMap::from_iter([
            (BrilligFunctionId(0), true),
            (BrilligFunctionId(1), true),
        ])));
        assert!(has_side_effect_free_brillig(&BTreeMap::from_iter([
            (BrilligFunctionId(0), true),
            (BrilligFunctionId(1), false),
        ])));
    }

    #[test]
    fn skips_reoptimization_without_side_effect_free_brillig() {
        let circuit = wide_circuit();
        let program = Program { functions: vec![circuit.clone()], unconstrained_functions: vec![] };
        let mut debug = vec![DebugInfo::default()];

        let optimized = optimize_program_internal(program, &mut debug);

        // With no Brillig call known to be side-effect free, re-optimizing cannot
        // improve on the pass already run during ACIR generation, so the circuit is
        // returned untouched rather than re-running the backend transform (which would
        // have split the wide expression into intermediate variables).
        assert_eq!(optimized.functions, vec![circuit]);
    }
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
