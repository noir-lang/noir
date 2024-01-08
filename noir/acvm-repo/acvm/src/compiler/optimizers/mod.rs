use acir::circuit::{Circuit, Opcode};

mod general;
mod redundant_range;
mod unused_memory;

pub(crate) use general::GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;

use self::unused_memory::UnusedMemoryOptimizer;

use super::{transform_assert_messages, AcirTransformationMap};

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] independent optimizations to a [`Circuit`].
pub fn optimize(acir: Circuit) -> (Circuit, AcirTransformationMap) {
    let (mut acir, new_opcode_positions) = optimize_internal(acir);

    let transformation_map = AcirTransformationMap::new(new_opcode_positions);

    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] independent optimizations to a [`Circuit`].
pub(super) fn optimize_internal(acir: Circuit) -> (Circuit, Vec<usize>) {
    log::trace!("Start circuit optimization");

    // General optimizer pass
    let opcodes: Vec<Opcode> = acir
        .opcodes
        .into_iter()
        .map(|opcode| {
            if let Opcode::Arithmetic(arith_expr) = opcode {
                Opcode::Arithmetic(GeneralOptimizer::optimize(arith_expr))
            } else {
                opcode
            }
        })
        .collect();
    let acir = Circuit { opcodes, ..acir };

    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    let acir_opcode_positions = (0..acir.opcodes.len()).collect();

    // Unused memory optimization pass
    let memory_optimizer = UnusedMemoryOptimizer::new(acir);
    let (acir, acir_opcode_positions) =
        memory_optimizer.remove_unused_memory_initializations(acir_opcode_positions);

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let (acir, acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(acir_opcode_positions);

    log::trace!("Finish circuit optimization");

    (acir, acir_opcode_positions)
}
