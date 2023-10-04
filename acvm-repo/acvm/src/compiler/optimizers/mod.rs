use acir::circuit::{Circuit, Opcode};

mod general;
mod redundant_range;

pub(crate) use general::GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;

use super::AcirTransformationMap;

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

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let (acir, acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(acir_opcode_positions);

    let transformation_map = AcirTransformationMap { acir_opcode_positions };

    (acir, transformation_map)
}
