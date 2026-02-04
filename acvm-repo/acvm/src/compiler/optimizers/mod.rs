use std::collections::BTreeMap;

use acir::{
    AcirField,
    circuit::{Circuit, Opcode, brillig::BrilligFunctionId},
};

mod common_subexpression;
mod general;
mod redundant_range;
mod unused_memory;

pub(crate) use general::GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;
use tracing::info;

use self::unused_memory::UnusedMemoryOptimizer;

use super::{AcirTransformationMap, transform_assert_messages};

/// Applies backend independent optimizations to a [`Circuit`].
pub fn optimize<F: AcirField>(
    acir: Circuit<F>,
    brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
) -> (Circuit<F>, AcirTransformationMap) {
    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    // For instance, here before any transformation, the old acir opcode positions have not changed.
    // So acir_opcode_positions = 0, 1,...,n-1, representing the index of the opcode in Circuit.opcodes vector.
    let acir_opcode_positions = (0..acir.opcodes.len()).collect();

    // `optimize_internal()` may change the circuit, and it returns a new one, as well the new_opcode_positions
    // In the new circuit, the opcode at index `i` corresponds to the opcode at index `new_opcode_positions[i]` in the original circuit.
    // For instance let's say it removed the opcode at index 3, and replaced the one at index 5 by two new opcodes
    // The new_opcode_positions is now: 0,1,2,4,5,5,6,....n-1
    let (mut acir, new_opcode_positions) =
        optimize_internal(acir, acir_opcode_positions, brillig_side_effects);

    let transformation_map = AcirTransformationMap::new(&new_opcode_positions);

    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}

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

    let max_transformer_passes_or_default = None;
    let (acir, acir_opcode_positions, _opcodes_hash_stabilized) =
        common_subexpression::transform_internal(
            acir,
            acir_opcode_positions,
            brillig_side_effects,
            max_transformer_passes_or_default,
        );

    info!("Number of opcodes after: {}", acir.opcodes.len());

    (acir, acir_opcode_positions)
}
