use std::collections::BTreeMap;

use acir::{
    AcirField,
    circuit::{Circuit, Opcode, brillig::BrilligFunctionId},
};

mod general;
mod merge_expressions;
mod redundant_range;
mod unused_memory;

pub(crate) use general::GeneralOptimizer;
pub(crate) use merge_expressions::MergeExpressionsOptimizer;
pub(crate) use redundant_range::RangeOptimizer;
use tracing::info;

use self::unused_memory::UnusedMemoryOptimizer;

/// Applies backend independent optimizations to a [`Circuit`].
///
/// Accepts an injected `acir_opcode_positions` to allow optimizations to be applied in a loop.
/// It run the following passes:
/// - General optimizer
/// - Unused Memory optimization
/// - Redundant Ranges optimization
#[tracing::instrument(level = "trace", name = "optimize_acir" skip(acir, acir_opcode_positions))]
pub(super) fn optimize_internal<F: AcirField>(
    acir: Circuit<F>,
    acir_opcode_positions: Vec<usize>,
    brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
) -> (Circuit<F>, Vec<usize>) {
    if acir.opcodes.len() == 1 && matches!(acir.opcodes[0], Opcode::BrilligCall { .. }) {
        info!("Program is fully unconstrained, skipping optimization pass");
        return (acir, acir_opcode_positions);
    }

    info!("Number of opcodes before: {}", acir.opcodes.len());

    // General optimizer pass
    let opcodes: Vec<Opcode<F>> = acir
        .opcodes
        .into_iter()
        .map(|opcode| {
            if let Opcode::AssertZero(arith_expr) = opcode {
                Opcode::AssertZero(GeneralOptimizer::optimize(arith_expr))
            } else {
                opcode
            }
        })
        .collect();
    let acir = Circuit { opcodes, ..acir };

    // Unused memory optimization pass
    let memory_optimizer = UnusedMemoryOptimizer::new(acir);
    let (acir, acir_opcode_positions) =
        memory_optimizer.remove_unused_memory_initializations(acir_opcode_positions);

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir, brillig_side_effects);
    let (acir, acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(acir_opcode_positions);

    info!("Number of opcodes after: {}", acir.opcodes.len());

    (acir, acir_opcode_positions)
}
