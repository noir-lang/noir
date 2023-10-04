use acir::circuit::{opcodes::BlockId, Circuit, Opcode};
use std::collections::HashSet;

/// `RangeOptimizer` will remove redundant range constraints.
///
/// # Example
///
/// Suppose we had the following pseudo-code:
///
/// ```noir
/// let z1 = x as u16;
/// let z2 = x as u32;
/// ```
/// It is clear that if `x` fits inside of a 16-bit integer,
/// it must also fit inside of a 32-bit integer.
///
/// The generated ACIR may produce two range opcodes however;
/// - One for the 16 bit range constraint of `x`
/// - One for the 32-bit range constraint of `x`
///
/// This optimization pass will keep the 16-bit range constraint
/// and remove the 32-bit range constraint opcode.
pub(crate) struct UnusedMemoryOptimizer {
    unused_memory_initializations: HashSet<BlockId>,
    circuit: Circuit,
}

impl UnusedMemoryOptimizer {
    /// Creates a new `RangeOptimizer` by collecting all known range
    /// constraints from `Circuit`.
    pub(crate) fn new(circuit: Circuit) -> Self {
        let unused_memory_initializations = Self::collect_unused_memory_initializations(&circuit);
        Self { circuit, unused_memory_initializations }
    }

    /// Stores the lowest bit range, that a witness
    /// has been constrained to be.
    /// For example, if we constrain a witness `x` to be
    /// both 32 bits and 16 bits. This function will
    /// only store the fact that we have constrained it to
    /// be 16 bits.
    fn collect_unused_memory_initializations(circuit: &Circuit) -> HashSet<BlockId> {
        let mut unused_memory_initialization = HashSet::new();

        for opcode in &circuit.opcodes {
            match opcode {
                Opcode::MemoryInit { block_id, .. } => {
                    unused_memory_initialization.insert(*block_id);
                }
                Opcode::MemoryOp { block_id, .. } => {
                    unused_memory_initialization.remove(block_id);
                }
                _ => (),
            }
        }
        unused_memory_initialization
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// once to the lowest number `bit size` possible.
    pub(crate) fn remove_unused_memory_initializations(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit, Vec<usize>) {
        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut optimized_opcodes = Vec::with_capacity(self.circuit.opcodes.len());
        for (idx, opcode) in self.circuit.opcodes.iter().enumerate() {
            match opcode {
                Opcode::MemoryInit { block_id, .. }
                    if self.unused_memory_initializations.contains(block_id) =>
                {
                    // Drop opcode
                }
                _ => {
                    new_order_list.push(order_list[idx]);
                    optimized_opcodes.push(opcode.clone());
                }
            }
        }

        (
            Circuit {
                current_witness_index: self.circuit.current_witness_index,
                opcodes: optimized_opcodes,
                ..self.circuit
            },
            new_order_list,
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::BTreeSet;

//     use crate::compiler::optimizers::redundant_range::{extract_range_opcode, RangeOptimizer};
//     use acir::{
//         circuit::{
//             opcodes::{BlackBoxFuncCall, FunctionInput},
//             Circuit, Opcode, PublicInputs,
//         },
//         native_types::{Expression, Witness},
//     };

//     fn test_circuit(ranges: Vec<(Witness, u32)>) -> Circuit {
//         fn test_range_constraint(witness: Witness, num_bits: u32) -> Opcode {
//             Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
//                 input: FunctionInput { witness, num_bits },
//             })
//         }

//         let opcodes: Vec<_> = ranges
//             .into_iter()
//             .map(|(witness, num_bits)| test_range_constraint(witness, num_bits))
//             .collect();

//         Circuit {
//             current_witness_index: 1,
//             opcodes,
//             private_parameters: BTreeSet::new(),
//             public_parameters: PublicInputs::default(),
//             return_values: PublicInputs::default(),
//             assert_messages: Default::default(),
//         }
//     }

//     #[test]
//     fn retain_lowest_range_size() {
//         // The optimizer should keep the lowest bit size range constraint
//         let circuit = test_circuit(vec![(Witness(1), 32), (Witness(1), 16)]);
//         let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
//         let optimizer = RangeOptimizer::new(circuit);

//         let range_size = *optimizer
//             .lists
//             .get(&Witness(1))
//             .expect("Witness(1) was inserted, but it is missing from the map");
//         assert_eq!(
//             range_size, 16,
//             "expected a range size of 16 since that was the lowest bit size provided"
//         );

//         let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
//         assert_eq!(optimized_circuit.opcodes.len(), 1);

//         let (witness, num_bits) =
//             extract_range_opcode(&optimized_circuit.opcodes[0]).expect("expected one range opcode");

//         assert_eq!(witness, Witness(1));
//         assert_eq!(num_bits, 16);
//     }

//     #[test]
//     fn remove_duplicates() {
//         // The optimizer should remove all duplicate range opcodes.

//         let circuit = test_circuit(vec![
//             (Witness(1), 16),
//             (Witness(1), 16),
//             (Witness(2), 23),
//             (Witness(2), 23),
//         ]);
//         let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
//         let optimizer = RangeOptimizer::new(circuit);
//         let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
//         assert_eq!(optimized_circuit.opcodes.len(), 2);

//         let (witness_a, num_bits_a) =
//             extract_range_opcode(&optimized_circuit.opcodes[0]).expect("expected two range opcode");
//         let (witness_b, num_bits_b) =
//             extract_range_opcode(&optimized_circuit.opcodes[1]).expect("expected two range opcode");

//         assert_eq!(witness_a, Witness(1));
//         assert_eq!(witness_b, Witness(2));
//         assert_eq!(num_bits_a, 16);
//         assert_eq!(num_bits_b, 23);
//     }

//     #[test]
//     fn non_range_opcodes() {
//         // The optimizer should not remove or change non-range opcodes
//         // The four Arithmetic opcodes should remain unchanged.
//         let mut circuit = test_circuit(vec![(Witness(1), 16), (Witness(1), 16)]);

//         circuit.opcodes.push(Opcode::Arithmetic(Expression::default()));
//         circuit.opcodes.push(Opcode::Arithmetic(Expression::default()));
//         circuit.opcodes.push(Opcode::Arithmetic(Expression::default()));
//         circuit.opcodes.push(Opcode::Arithmetic(Expression::default()));
//         let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
//         let optimizer = RangeOptimizer::new(circuit);
//         let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
//         assert_eq!(optimized_circuit.opcodes.len(), 5);
//     }
// }
