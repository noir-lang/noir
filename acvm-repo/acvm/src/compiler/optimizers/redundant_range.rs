//! The redundant range constraint optimization pass aims to remove any [BlackBoxFunc::Range] opcodes
//! which doesn't result in additional restrictions on the value of witnesses.
//!
//! Suppose we had the following pseudo-code:
//!
//! ```noir
//! let z1 = x as u16;
//! let z2 = x as u32;
//! ```
//! It is clear that if `x` fits inside of a 16-bit integer,
//! it must also fit inside of a 32-bit integer.
//!
//! The generated ACIR may produce two range opcodes however;
//! - One for the 16 bit range constraint of `x`
//! - One for the 32-bit range constraint of `x`
//!
//! This optimization pass will keep the 16-bit range constraint
//! and remove the 32-bit range constraint opcode.
//!
//! # Implicit range constraints
//!
//! We also consider implicit range constraints on witnesses - constraints other than [BlackBoxFunc::Range]
//! which limit the size of a witness.
//!
//! ## Constant assignments
//!
//! The most obvious of these are when a witness is constrained to be equal to a constant value.
//!
//! ```noir
//! let z1 = x as u16;
//! assert_eq(z1, 100);
//! ```
//!
//! We can consider the assertion that `z1 == 100` to be equivalent to a range constraint for `z1` to fit within
//! 7 bits (the minimum necessary to hold the value `100`).
//!
//! ## Array indexing
//!
//! Another situation which adds an implicit range constraint are array indexing, for example in the program:
//!
//! ```noir
//! fn main(index: u32) -> pub Field {
//!     let array: [Field; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
//!     array[index]
//! }
//! ```
//!
//! Here the variable `index` is range constrained to fit within 32 bits by the `u32` type however
//! it's constrained more restrictively by the length of `array`. If `index` were 10 or greater then
//! it would result in a read past the end of the array, which is invalid. We can then remove the explicit
//! range constraint on `index` as the usage as an array index more tightly constrains its value.
//!
//! [BlackBoxFunc::Range]: acir::circuit::black_box_functions::BlackBoxFunc::RANGE

use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
    },
    native_types::Witness,
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) struct RangeOptimizer<F: AcirField> {
    /// Maps witnesses to their lowest known bit sizes.
    lists: BTreeMap<Witness, u32>,
    circuit: Circuit<F>,
}

impl<F: AcirField> RangeOptimizer<F> {
    /// Creates a new `RangeOptimizer` by collecting all known range
    /// constraints from `Circuit`.
    pub(crate) fn new(circuit: Circuit<F>) -> Self {
        let range_list = Self::collect_ranges(&circuit);
        Self { circuit, lists: range_list }
    }

    /// Stores the lowest bit range, that a witness
    /// has been constrained to be.
    /// For example, if we constrain a witness `x` to be
    /// both 32 bits and 16 bits. This function will
    /// only store the fact that we have constrained it to
    /// be 16 bits.
    fn collect_ranges(circuit: &Circuit<F>) -> BTreeMap<Witness, u32> {
        let mut witness_to_bit_sizes: BTreeMap<Witness, u32> = BTreeMap::new();
        let mut memory_block_lengths_bit_size: HashMap<BlockId, u32> = HashMap::new();

        for opcode in &circuit.opcodes {
            let Some((witness, num_bits)) = (match opcode {
                Opcode::AssertZero(expr) => {
                    // If the opcode is constraining a witness to be equal to a value then it can be considered
                    // as a range opcode for the number of bits required to hold that value.
                    if expr.is_degree_one_univariate() {
                        let (k, witness) = expr.linear_combinations[0];
                        let constant = expr.q_c;
                        let witness_value = -constant / k;

                        if witness_value.is_zero() {
                            Some((witness, 0))
                        } else {
                            // We subtract off 1 bit from the implied witness value to give the weakest range constraint
                            // which would be stricter than the constraint imposed by this opcode.
                            let implied_range_constraint_bits = witness_value.num_bits() - 1;
                            Some((witness, implied_range_constraint_bits))
                        }
                    } else {
                        None
                    }
                }

                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input, num_bits }) => {
                    if let FunctionInput::Witness(witness) = input {
                        Some((*witness, *num_bits))
                    } else {
                        None
                    }
                }

                Opcode::MemoryInit { block_id, init, .. } => {
                    memory_block_lengths_bit_size
                        .insert(*block_id, memory_block_implied_max_bits(init));
                    None
                }

                Opcode::MemoryOp { block_id, op: MemOp { index, .. }, .. } => {
                    index.to_witness().map(|witness| {
                        (
                            witness,
                            *memory_block_lengths_bit_size
                                .get(block_id)
                                .expect("memory must be initialized before any reads/writes"),
                        )
                    })
                }

                _ => None,
            }) else {
                continue;
            };

            // Check if the witness has already been recorded and if the witness
            // size is more than the current one, we replace it
            witness_to_bit_sizes
                .entry(witness)
                .and_modify(|old_range_bits| {
                    *old_range_bits = std::cmp::min(*old_range_bits, num_bits);
                })
                .or_insert(num_bits);
        }
        witness_to_bit_sizes
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// once to the lowest number `bit size` possible.
    pub(crate) fn replace_redundant_ranges(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let mut already_seen_witness = HashSet::new();

        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut optimized_opcodes = Vec::with_capacity(self.circuit.opcodes.len());
        for (idx, opcode) in self.circuit.opcodes.into_iter().enumerate() {
            let (witness, num_bits) = {
                // If its not the range opcode, add it to the opcode
                // list and continue;
                let mut push_non_range_opcode = || {
                    optimized_opcodes.push(opcode.clone());
                    new_order_list.push(order_list[idx]);
                };

                match opcode {
                    Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                        input: FunctionInput::Witness(witness),
                        num_bits,
                    }) => (witness, num_bits),
                    _ => {
                        push_non_range_opcode();
                        continue;
                    }
                }
            };
            // If we've already applied the range constraint for this witness then skip this opcode.
            let already_added = already_seen_witness.contains(&witness);
            if already_added {
                continue;
            }

            // Check if this is the lowest number of bits in the circuit
            let stored_num_bits = self.lists.get(&witness).expect("Could not find witness. This should never be the case if `collect_ranges` is called");
            let is_lowest_bit_size = num_bits <= *stored_num_bits;

            // If the opcode is associated with the lowest bit size
            // and we have not added a duplicate of this opcode yet,
            // then we should add retain this range opcode.
            if is_lowest_bit_size {
                already_seen_witness.insert(witness);
                new_order_list.push(order_list[idx]);
                optimized_opcodes.push(opcode.clone());
            }
        }

        (Circuit { opcodes: optimized_opcodes, ..self.circuit }, new_order_list)
    }
}

fn memory_block_implied_max_bits(init: &[Witness]) -> u32 {
    let array_len = init.len() as u32;
    32 - array_len.leading_zeros()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::compiler::optimizers::redundant_range::{
        RangeOptimizer, memory_block_implied_max_bits,
    };
    use acir::{
        FieldElement,
        circuit::{
            Circuit, ExpressionWidth, Opcode, PublicInputs,
            opcodes::{BlackBoxFuncCall, BlockId, BlockType, FunctionInput, MemOp},
        },
        native_types::{Expression, Witness},
    };

    #[test]
    fn correctly_calculates_memory_block_implied_max_bits() {
        assert_eq!(memory_block_implied_max_bits(&[]), 0);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 1]), 1);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 2]), 2);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 3]), 2);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 4]), 3);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 8]), 4);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); u8::MAX as usize]), 8);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); u16::MAX as usize]), 16);
    }

    fn test_circuit(ranges: Vec<(Witness, u32)>) -> Circuit<FieldElement> {
        fn test_range_constraint(witness: Witness, num_bits: u32) -> Opcode<FieldElement> {
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(witness),
                num_bits,
            })
        }

        let opcodes: Vec<_> = ranges
            .into_iter()
            .map(|(witness, num_bits)| test_range_constraint(witness, num_bits))
            .collect();

        Circuit {
            name: "test".to_string(),
            current_witness_index: 1,
            opcodes,
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        }
    }

    #[test]
    fn retain_lowest_range_size() {
        // The optimizer should keep the lowest bit size range constraint
        let circuit = test_circuit(vec![(Witness(1), 32), (Witness(1), 16)]);
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit);

        let range_size = *optimizer
            .lists
            .get(&Witness(1))
            .expect("Witness(1) was inserted, but it is missing from the map");
        assert_eq!(
            range_size, 16,
            "expected a range size of 16 since that was the lowest bit size provided"
        );

        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 1);

        assert_eq!(
            optimized_circuit.opcodes[0],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(Witness(1)),
                num_bits: 16,
            })
        );
    }

    #[test]
    fn remove_duplicates() {
        // The optimizer should remove all duplicate range opcodes.

        let circuit = test_circuit(vec![
            (Witness(1), 16),
            (Witness(1), 16),
            (Witness(2), 23),
            (Witness(2), 23),
        ]);
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 2);

        assert_eq!(
            optimized_circuit.opcodes[0],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(Witness(1)),
                num_bits: 16,
            })
        );
        assert_eq!(
            optimized_circuit.opcodes[1],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(Witness(2)),
                num_bits: 23,
            })
        );
    }

    #[test]
    fn non_range_opcodes() {
        // The optimizer should not remove or change non-range opcodes
        // The four AssertZero opcodes should remain unchanged.
        let mut circuit = test_circuit(vec![(Witness(1), 16), (Witness(1), 16)]);

        circuit.opcodes.push(Opcode::AssertZero(Expression::default()));
        circuit.opcodes.push(Opcode::AssertZero(Expression::default()));
        circuit.opcodes.push(Opcode::AssertZero(Expression::default()));
        circuit.opcodes.push(Opcode::AssertZero(Expression::default()));
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 5);
    }

    #[test]
    fn constant_implied_ranges() {
        // The optimizer should use knowledge about constant witness assignments to remove range opcodes.
        let mut circuit = test_circuit(vec![(Witness(1), 16)]);

        circuit.opcodes.push(Opcode::AssertZero(Witness(1).into()));
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 1);
        assert_eq!(optimized_circuit.opcodes[0], Opcode::AssertZero(Witness(1).into()));
    }

    #[test]
    fn array_implied_ranges() {
        // The optimizer should use knowledge about array lengths and witnesses used to index these to remove range opcodes.
        let mut circuit = test_circuit(vec![(Witness(1), 16)]);

        let mem_init = Opcode::MemoryInit {
            block_id: BlockId(0),
            init: vec![Witness(0); 8],
            block_type: BlockType::Memory,
        };
        let mem_op = Opcode::MemoryOp {
            block_id: BlockId(0),
            op: MemOp::read_at_mem_index(Witness(1).into(), Witness(2)),
        };

        circuit.opcodes.push(mem_init.clone());
        circuit.opcodes.push(mem_op.clone());
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 2);
        assert_eq!(optimized_circuit.opcodes[0], mem_init);
        assert_eq!(optimized_circuit.opcodes[1], mem_op);
    }
}
