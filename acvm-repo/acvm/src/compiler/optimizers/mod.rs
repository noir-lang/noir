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
    // General optimizer pass
    let mut opcodes: Vec<Opcode> = Vec::new();
    for opcode in acir.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                opcodes.push(Opcode::Arithmetic(GeneralOptimizer::optimize(arith_expr)));
            }
            other_opcode => opcodes.push(other_opcode),
        };
    }
    let acir = Circuit { opcodes, ..acir };

    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    let acir_opcode_positions = acir.opcodes.iter().enumerate().map(|(i, _)| i).collect();

    // Unused memory optimization pass
    let memory_optimizer = UnusedMemoryOptimizer::new(acir);
    let (acir, acir_opcode_positions) =
        memory_optimizer.remove_unused_memory_initializations(acir_opcode_positions);

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let (mut acir, acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(acir_opcode_positions);

    let transformation_map = AcirTransformationMap { acir_opcode_positions };

    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}
