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
//! # Side effects
//!
//! The pass will keep range constraints where, should the constraint have failed, removing it
//! would allow potentially side effecting Brillig calls to be executed, before another constraint
//! further down the line would have stopped the circuit.
//!
//! [BlackBoxFunc::Range]: acir::circuit::black_box_functions::BlackBoxFunc::RANGE

use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        brillig::BrilligFunctionId,
        opcodes::{BlackBoxFuncCall, BlockId, ConstantOrWitnessEnum, FunctionInput, MemOp},
    },
    native_types::Witness,
};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Information gathered about witnesses which are subject to range constraints.
struct RangeInfo {
    /// The minimum overall explicit or implied bit size.
    min_num_bits: u32,
    /// Indicate that the minimum bit size comes from an assertion, in which case we don't need to keep the range constraint.
    min_num_bits_is_implied: bool,
    /// The minimum opcode position at which the minimum bit size has been recorded.
    min_num_bits_idx: usize,
    /// The minimum opcode index after the range constraint which might have a side effect.
    min_side_effect_idx: Option<usize>,
}

pub(crate) struct RangeOptimizer<F: AcirField> {
    /// Maps witnesses to their lowest known bit sizes.
    infos: BTreeMap<Witness, RangeInfo>,
    circuit: Circuit<F>,
}

impl<F: AcirField> RangeOptimizer<F> {
    /// Creates a new `RangeOptimizer` by collecting all known range
    /// constraints from `Circuit`.
    pub(crate) fn new(
        circuit: Circuit<F>,
        brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
    ) -> Self {
        let next_side_effects = Self::collect_side_effects(&circuit, brillig_side_effects);
        let infos = Self::collect_ranges(&circuit, next_side_effects);
        Self { circuit, infos }
    }

    /// Stores the lowest bit range, that a witness
    /// has been constrained to be.
    /// For example, if we constrain a witness `x` to be
    /// both 32 bits and 16 bits. This function will
    /// only store the fact that we have constrained it to
    /// be 16 bits.
    fn collect_ranges(
        circuit: &Circuit<F>,
        next_side_effects: Vec<Option<usize>>,
    ) -> BTreeMap<Witness, RangeInfo> {
        let mut infos: BTreeMap<Witness, RangeInfo> = BTreeMap::new();
        let mut memory_block_lengths_bit_size: HashMap<BlockId, u32> = HashMap::new();

        for (idx, opcode) in circuit.opcodes.iter().enumerate() {
            let Some((witness, num_bits, is_implied)) = (match opcode {
                Opcode::AssertZero(expr) => {
                    // If the opcode is constraining a witness to be equal to a value then it can be considered
                    // as a range opcode for the number of bits required to hold that value.
                    if expr.is_degree_one_univariate() {
                        let (k, witness) = expr.linear_combinations[0];
                        let constant = expr.q_c;
                        let witness_value = -constant / k;

                        if witness_value.is_zero() {
                            Some((witness, 0, true))
                        } else {
                            let implied_range_constraint_bits = witness_value.num_bits();
                            Some((witness, implied_range_constraint_bits, true))
                        }
                    } else {
                        None
                    }
                }

                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input }) => {
                    if let ConstantOrWitnessEnum::Witness(witness) = input.input() {
                        Some((witness, input.num_bits(), false))
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
                            false,
                        )
                    })
                }

                _ => None,
            }) else {
                continue;
            };

            let next_side_effect = next_side_effects[idx];

            // Check if the witness has already been recorded and if the witness
            // size is more than the current one, we replace it
            infos
                .entry(witness)
                .and_modify(|info| {
                    if num_bits < info.min_num_bits {
                        info.min_num_bits = num_bits;
                        info.min_num_bits_is_implied = is_implied;
                        info.min_num_bits_idx = idx;
                    } else if num_bits == info.min_num_bits
                        && is_implied
                        && !info.min_num_bits_is_implied
                    {
                        info.min_num_bits_is_implied = true;
                        info.min_num_bits_idx = idx;
                    }
                })
                .or_insert_with(|| RangeInfo {
                    min_num_bits: num_bits,
                    min_num_bits_is_implied: is_implied,
                    min_num_bits_idx: idx,
                    min_side_effect_idx: next_side_effect,
                });
        }
        infos
    }

    /// Return a vector of the next potential side effect for each opcode position.
    fn collect_side_effects(
        circuit: &Circuit<F>,
        brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
    ) -> Vec<Option<usize>> {
        let mut output = Vec::new();
        let mut last_side_effect = None;

        // Go in reverse so we can propagate the information backwards.
        for (idx, opcode) in circuit.opcodes.iter().enumerate().rev() {
            match opcode {
                // Assume that Brillig calls might have side effects, unless we know they don't.
                Opcode::BrilligCall { id, .. }
                    if brillig_side_effects.get(id).copied().unwrap_or(true) =>
                {
                    last_side_effect = Some(idx);
                }
                // Not sure if ACIR calls should be marked. For now only doing it for Brillig.
                _ => {}
            }
            output.push(last_side_effect);
        }

        output.reverse();
        output
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// at most once to the lowest number `bit size` possible.
    pub(crate) fn replace_redundant_ranges(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let mut already_seen_witness = HashSet::new();

        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut optimized_opcodes = Vec::with_capacity(self.circuit.opcodes.len());
        for (idx, opcode) in self.circuit.opcodes.into_iter().enumerate() {
            let Some((witness, num_bits)) = (match opcode {
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input }) => {
                    match input.input() {
                        ConstantOrWitnessEnum::Witness(witness) => {
                            Some((witness, input.num_bits()))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }) else {
                // If its not the range opcode, add it to the opcode list and continue.
                optimized_opcodes.push(opcode.clone());
                new_order_list.push(order_list[idx]);
                continue;
            };

            // If we've already applied the range constraint for this witness then skip this opcode.
            if already_seen_witness.contains(&witness) {
                continue;
            }

            let info = self.infos.get(&witness).expect("Could not find witness. This should never be the case if `collect_ranges` is called");

            // Decide if this is the time to insert the one range constraint we might want to keep.
            let keep=
                // If the minimum bits comes from a range constraint, we should keep it, but if it's implied we can let the assert do it and save an opcode.
                num_bits == info.min_num_bits && !info.min_num_bits_is_implied ||
                // If there is a side effect the witness participates in, and it comes before where the minimum bit size would be enforced, insert a constraint as early as possible.
                info.min_side_effect_idx.is_some_and(|min_side_effect_idx| min_side_effect_idx < info.min_num_bits_idx);

            if keep {
                // Keep the first range constraint, but use the lowest stored bit size for it.
                let opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::witness(witness, info.min_num_bits),
                });
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
    use std::collections::{BTreeMap, BTreeSet};

    use crate::compiler::optimizers::redundant_range::{
        RangeOptimizer, memory_block_implied_max_bits,
    };
    use acir::{
        FieldElement,
        circuit::{
            Circuit, ExpressionWidth, Opcode, PublicInputs,
            brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
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
                input: FunctionInput::witness(witness, num_bits),
            })
        }

        let opcodes: Vec<_> = ranges
            .into_iter()
            .map(|(witness, num_bits)| test_range_constraint(witness, num_bits))
            .collect();

        Circuit {
            current_witness_index: 1,
            expression_width: ExpressionWidth::Bounded { width: 4 },
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
        let optimizer = RangeOptimizer::new(circuit, &Default::default());

        let info = optimizer
            .infos
            .get(&Witness(1))
            .expect("Witness(1) was inserted, but it is missing from the map");
        assert_eq!(
            info.min_num_bits, 16,
            "expected a range size of 16 since that was the lowest bit size provided"
        );

        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 1);

        assert_eq!(
            optimized_circuit.opcodes[0],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(1), 16)
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
        let optimizer = RangeOptimizer::new(circuit, &Default::default());
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 2);

        assert_eq!(
            optimized_circuit.opcodes[0],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(1), 16)
            })
        );
        assert_eq!(
            optimized_circuit.opcodes[1],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(2), 23)
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
        let optimizer = RangeOptimizer::new(circuit, &Default::default());
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 5);
    }

    #[test]
    fn constant_implied_ranges() {
        // The optimizer should use knowledge about constant witness assignments to remove range opcodes.
        let mut circuit = test_circuit(vec![(Witness(1), 16)]);

        circuit.opcodes.push(Opcode::AssertZero(Witness(1).into()));
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit, &Default::default());
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 1);
        assert_eq!(optimized_circuit.opcodes[0], Opcode::AssertZero(Witness(1).into()));
    }

    #[test]
    fn potential_side_effects() {
        // The optimizer should not remove range constraints if doing so might allow invalid side effects to go through.
        let mut circuit = test_circuit(vec![(Witness(1), 16)]);

        // Call brillig with w2
        circuit.opcodes.push(Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![BrilligInputs::Single(Witness(2).into())],
            outputs: vec![BrilligOutputs::Simple(Witness(3))],
            predicate: None,
        });

        // assert w1 == 0
        circuit.opcodes.push(Opcode::AssertZero(Witness(1).into()));

        let acir_opcode_positions: Vec<usize> =
            circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();

        // All opcodes are expected to be kept.
        let expected_length = acir_opcode_positions.len();

        // Consider the Brillig function to have a side effect.
        let brillig_side_effects = BTreeMap::from_iter(vec![(BrilligFunctionId(0), true)]);

        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) =
            optimizer.replace_redundant_ranges(acir_opcode_positions.clone());

        assert_eq!(optimized_circuit.opcodes.len(), expected_length);
        assert_eq!(
            optimized_circuit.opcodes[0],
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(1), 0) // number of bits implied from the constant
            })
        );

        // Applying again should have no effect (despite the range having the same bit size as the assert).
        let optimizer = RangeOptimizer::new(optimized_circuit, &brillig_side_effects);
        let (double_optimized_circuit, _) =
            optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(double_optimized_circuit.opcodes.len(), expected_length);
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
            predicate: None,
        };

        circuit.opcodes.push(mem_init.clone());
        circuit.opcodes.push(mem_op.clone());
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = RangeOptimizer::new(circuit, &Default::default());
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.opcodes.len(), 2);
        assert_eq!(optimized_circuit.opcodes[0], mem_init);
        assert_eq!(optimized_circuit.opcodes[1], mem_op);
    }
}
