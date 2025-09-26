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
        opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
    },
    native_types::Witness,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Information gathered about witnesses which are subject to range constraints.
struct RangeInfo {
    /// Opcode positions at which stricter bit size information becomes available.
    switch_points: BTreeSet<usize>,
    /// Strictest constraint on bit size so far.
    num_bits: u32,
    /// Indicate whether the bit size comes from an assertion, in which case we
    /// can save an equivalent range constraint.
    is_implied: bool,
}

pub(crate) struct RangeOptimizer<'a, F: AcirField> {
    /// Maps witnesses to their bit size switch points.
    infos: BTreeMap<Witness, RangeInfo>,
    /// The next potential side effect for each opcode.
    brillig_side_effects: &'a BTreeMap<BrilligFunctionId, bool>,
    circuit: Circuit<F>,
}

impl<'a, F: AcirField> RangeOptimizer<'a, F> {
    /// Creates a new `RangeOptimizer` by collecting all known range
    /// constraints from `Circuit`.
    pub(crate) fn new(
        circuit: Circuit<F>,
        brillig_side_effects: &'a BTreeMap<BrilligFunctionId, bool>,
    ) -> Self {
        let infos = Self::collect_ranges(&circuit);
        Self { circuit, infos, brillig_side_effects }
    }

    /// Collect range information about witnesses.
    fn collect_ranges(circuit: &Circuit<F>) -> BTreeMap<Witness, RangeInfo> {
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

                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input, num_bits }) => {
                    if let FunctionInput::Witness(witness) = input {
                        Some((*witness, *num_bits, false))
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
                            true,
                        )
                    })
                }

                _ => None,
            }) else {
                continue;
            };

            // Check if the witness has already been recorded and if the witness
            // size is more than the current one, we replace it
            infos
                .entry(witness)
                .and_modify(|info| {
                    if num_bits < info.num_bits
                        || num_bits == info.num_bits && is_implied && !info.is_implied
                    {
                        info.switch_points.insert(idx);
                        info.num_bits = num_bits;
                        info.is_implied = is_implied;
                    }
                })
                .or_insert_with(|| RangeInfo {
                    num_bits,
                    is_implied,
                    switch_points: BTreeSet::from_iter(std::iter::once(idx)),
                });
        }
        infos
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// a minimal number of times that still allows us to avoid executing
    /// any new side effects due to their removal.
    pub(crate) fn replace_redundant_ranges(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut optimized_opcodes = Vec::with_capacity(self.circuit.opcodes.len());
        // Consider the index beyond the last as a pseudo size effect by which time all constraints need to be inserted.
        let mut next_side_effect = self.circuit.opcodes.len();
        // Going in reverse so we can propagate the side effect information backwards.
        for (idx, opcode) in self.circuit.opcodes.into_iter().enumerate().rev() {
            let Some(witness) = (match opcode {
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(witness),
                    ..
                }) => Some(witness),
                Opcode::BrilligCall { id, .. } => {
                    // Assume that Brillig calls might have side effects, unless we know they don't.
                    if self.brillig_side_effects.get(&id).copied().unwrap_or(true) {
                        next_side_effect = idx;
                    }
                    None
                }
                _ => None,
            }) else {
                // If its not the range opcode, add it to the opcode list and continue.
                optimized_opcodes.push(opcode.clone());
                new_order_list.push(order_list[idx]);
                continue;
            };

            let info = self.infos.get(&witness).expect("Could not find witness. This should never be the case if `collect_ranges` is called");

            // If this is not a switch point, then we should have already added a range constraint at least as strict, if it was needed.
            if !info.switch_points.contains(&idx) {
                continue;
            }

            // Check if we have an even stricter point before the next side effect.
            let has_stricter_before_next_side_effect = info
                .switch_points
                .iter()
                .any(|switch_idx| *switch_idx > idx && *switch_idx < next_side_effect);

            // If there is something even stricter before the next side effect (or the end), we don't need this.
            if has_stricter_before_next_side_effect {
                continue;
            }

            new_order_list.push(order_list[idx]);
            optimized_opcodes.push(opcode.clone());
        }

        // Restore forward order.
        optimized_opcodes.reverse();
        new_order_list.reverse();

        (Circuit { opcodes: optimized_opcodes, ..self.circuit }, new_order_list)
    }
}

/// Calculate the maximum number of bits required to index a memory block of a certain size.
fn memory_block_implied_max_bits(init: &[Witness]) -> u32 {
    let array_len = init.len() as u32;
    let max_index = array_len.saturating_sub(1);
    32 - max_index.leading_zeros()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        assert_circuit_snapshot,
        compiler::optimizers::redundant_range::{RangeOptimizer, memory_block_implied_max_bits},
    };
    use acir::{
        circuit::{Circuit, brillig::BrilligFunctionId},
        native_types::Witness,
    };

    #[test]
    fn correctly_calculates_memory_block_implied_max_bits() {
        assert_eq!(memory_block_implied_max_bits(&[]), 0);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 1]), 0);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 2]), 1);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 3]), 2);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 4]), 2);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); 8]), 3);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); u8::MAX as usize]), 8);
        assert_eq!(memory_block_implied_max_bits(&[Witness(0); u16::MAX as usize]), 16);
    }

    #[test]
    fn retain_lowest_range_size() {
        // The optimizer should keep the lowest bit size range constraint
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:32 bits []
        BLACKBOX::RANGE [w1]:16 bits []
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let brillig_side_effects = BTreeMap::new();
        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);

        let info = optimizer
            .infos
            .get(&Witness(1))
            .expect("Witness(1) was inserted, but it is missing from the map");
        assert_eq!(
            info.num_bits, 16,
            "expected a range size of 16 since that was the lowest bit size provided"
        );

        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        ");
    }

    #[test]
    fn remove_duplicates() {
        // The optimizer should remove all duplicate range opcodes.
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        BLACKBOX::RANGE [w1]:16 bits []
        BLACKBOX::RANGE [w2]:23 bits []
        BLACKBOX::RANGE [w2]:23 bits []
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let brillig_side_effects = BTreeMap::new();
        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        BLACKBOX::RANGE [w2]:23 bits []
        ");
    }

    #[test]
    fn non_range_opcodes() {
        // The optimizer should not remove or change non-range opcodes
        // The four AssertZero opcodes should remain unchanged.
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        BLACKBOX::RANGE [w1]:16 bits []
        EXPR 0 = 0
        EXPR 0 = 0
        EXPR 0 = 0
        EXPR 0 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let brillig_side_effects = BTreeMap::new();
        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        EXPR 0 = 0
        EXPR 0 = 0
        EXPR 0 = 0
        EXPR 0 = 0
        ");
    }

    #[test]
    fn constant_implied_ranges() {
        // The optimizer should use knowledge about constant witness assignments to remove range opcodes.
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        EXPR w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let brillig_side_effects = BTreeMap::new();
        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        EXPR w1 = 0
        ");
    }

    #[test]
    fn potential_side_effects() {
        // The optimizer should not remove range constraints if doing so might allow invalid side effects to go through.
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:32 bits []

        // Call brillig with w2
        BRILLIG CALL func: 0, inputs: [w2], outputs: []
        BLACKBOX::RANGE [w1]:16 bits []

        // Another call
        BRILLIG CALL func: 0, inputs: [w2], outputs: []

        // One more constraint, but this is redundant.
        BLACKBOX::RANGE [w1]:64 bits []

        // assert w1 == 0
        EXPR w1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions: Vec<usize> =
            circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();

        // Consider the Brillig function to have a side effect.
        let brillig_side_effects = BTreeMap::from_iter(vec![(BrilligFunctionId(0), true)]);

        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) =
            optimizer.replace_redundant_ranges(acir_opcode_positions.clone());

        // `BLACKBOX::RANGE [w1]:32 bits []` remains: The minimum does not propagate backwards.
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:32 bits []
        BRILLIG CALL func: 0, inputs: [w2], outputs: []
        BLACKBOX::RANGE [w1]:16 bits []
        BRILLIG CALL func: 0, inputs: [w2], outputs: []
        EXPR w1 = 0
        ");

        // Applying again should have no effect (despite the range having the same bit size as the assert).
        let optimizer = RangeOptimizer::new(optimized_circuit.clone(), &brillig_side_effects);
        let (double_optimized_circuit, _) =
            optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_eq!(optimized_circuit.to_string(), double_optimized_circuit.to_string());
    }

    #[test]
    fn array_implied_ranges() {
        // The optimizer should use knowledge about array lengths and witnesses used to index these to remove range opcodes.
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::RANGE [w1]:16 bits []
        INIT id: 0, len: 8, witnesses: [w0, w0, w0, w0, w0, w0, w0, w0]
        MEM id: 0, read at: w1, value: w2
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let brillig_side_effects = BTreeMap::new();
        let optimizer = RangeOptimizer::new(circuit, &brillig_side_effects);
        let (optimized_circuit, _) = optimizer.replace_redundant_ranges(acir_opcode_positions);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: []
        public parameters: []
        return values: []
        INIT id: 0, len: 8, witnesses: [w0, w0, w0, w0, w0, w0, w0, w0]
        MEM id: 0, read at: w1, value: w2
        ");
    }
}
