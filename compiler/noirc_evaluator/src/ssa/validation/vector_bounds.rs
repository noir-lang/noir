//! Validates that every `vector_insert`, `vector_remove`, and Brillig `vector_pop_back` /
//! `vector_pop_front` is preceded, in the same block, by a bounds check.
//!
//! SSA generation emits that check right before each call (see `codegen_intrinsic_call_checks`):
//!
//! | call                                           | check          |
//! |------------------------------------------------|----------------|
//! | `vector_insert(len, vector, index, ..)`        | `index <= len` |
//! | `vector_remove(len, vector, index)`            | `index < len`  |
//! | `vector_pop_back/front(len, vector)` (Brillig) | `0 < len`      |
//!
//! ACIR pops have no check: ACIR generation handles popping from an empty vector itself.
//!
//! The builder simplifies the check as it is inserted, so the call is accepted when its
//! requirement follows from constants or from one of these shapes earlier in the block:
//!
//! - `constrain (lt index, bound) == u1 1`, where `bound` is the length, `add len, u32 1` for an
//!   insert, or a constant that keeps the index in range. For a pop, `constrain (lt c, len) == u1 1`
//!   for a constant `c`.
//! - `range_check index to k bits` (possibly on `cast index as Field`), with `2^k` in range. This
//!   is how ACIR checks an index against a power-of-two bound.
//! - `constrain index == c` (possibly on `cast index as Field`), which is what `lt index, u32 1`
//!   and a range check to 0 bits simplify to.
//! - `constrain (eq len, u32 0) == u1 0`, which is what ACIR simplifies `lt u32 0, len` to, for
//!   a remove at index 0.
//! - A constraint between two different constants. It always fails, so the call never runs.
//!
//! Optimization passes can move the call away from its check or prove it in other ways (for
//! example flattening predicates the constraint), so this only runs in the full validation mode.
use acvm::AcirField;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, Intrinsic},
    value::{Value, ValueId},
};

/// Panics if a vector intrinsic in `function` is not preceded by its bounds check in its block.
/// See the module documentation for the accepted shapes.
pub(super) fn validate_vector_bounds_checks(function: &Function) {
    let dfg = &function.dfg;
    for block in function.reachable_blocks() {
        let mut checks = BlockChecks::default();
        for instruction in dfg[block].instructions() {
            match &dfg[*instruction] {
                Instruction::Constrain(lhs, rhs, _) => checks.record_constrain(dfg, *lhs, *rhs),
                Instruction::RangeCheck { value, max_bit_size, .. } => {
                    checks.record_range_check(dfg, *value, *max_bit_size);
                }
                Instruction::Call { func, arguments } => {
                    let Value::Intrinsic(intrinsic) = dfg[*func] else {
                        continue;
                    };
                    let length = arguments[0];
                    let checked = match intrinsic {
                        Intrinsic::VectorInsert => {
                            checks.index_in_bounds(dfg, arguments[2], length, true)
                        }
                        Intrinsic::VectorRemove => {
                            checks.index_in_bounds(dfg, arguments[2], length, false)
                        }
                        Intrinsic::VectorPopBack | Intrinsic::VectorPopFront
                            if function.runtime().is_brillig() =>
                        {
                            checks.is_non_empty(dfg, length)
                        }
                        _ => continue,
                    };
                    assert!(
                        checked,
                        "{intrinsic} call in function {} is not preceded by a bounds check",
                        function.id(),
                    );
                }
                _ => (),
            }
        }
    }
}

/// What the constraints and range checks seen so far in a block assert.
#[derive(Default)]
struct BlockChecks {
    /// Whether a constraint that always fails has been seen.
    always_fails: bool,
    /// `a -> [b]`: `a < b`.
    less_than: HashMap<ValueId, Vec<ValueId>>,
    /// `value -> bits`: `value < 2^bits`.
    bit_bounds: HashMap<ValueId, u32>,
    /// `value -> c`: `value == c`.
    equal_to: HashMap<ValueId, u128>,
    /// Values that are not zero.
    non_zero: HashSet<ValueId>,
}

impl BlockChecks {
    fn record_constrain(&mut self, dfg: &DataFlowGraph, lhs: ValueId, rhs: ValueId) {
        let (value, constant) = match (constant_value(dfg, lhs), constant_value(dfg, rhs)) {
            (None, Some(constant)) => (lhs, constant),
            (Some(constant), None) => (rhs, constant),
            (Some(lhs), Some(rhs)) => {
                self.always_fails |= lhs != rhs;
                return;
            }
            (None, None) => return,
        };
        self.equal_to.insert(value, constant);

        let Value::Instruction { instruction, .. } = &dfg[value] else {
            return;
        };
        match (&dfg[*instruction], constant) {
            (Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Lt }), 1) => {
                self.less_than.entry(*lhs).or_default().push(*rhs);
                // `c < rhs` for an unsigned constant `c` means `rhs` is not zero.
                if constant_value(dfg, *lhs).is_some() {
                    self.non_zero.insert(*rhs);
                }
            }
            (Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Eq }), 0)
                if constant_value(dfg, *rhs) == Some(0) =>
            {
                self.non_zero.insert(*lhs);
            }
            (Instruction::Cast(input, _), _) => {
                self.equal_to.insert(*input, constant);
            }
            _ => (),
        }
    }

    fn record_range_check(&mut self, dfg: &DataFlowGraph, value: ValueId, bits: u32) {
        let mut record = |value| {
            let bound = self.bit_bounds.entry(value).or_insert(bits);
            *bound = (*bound).min(bits);
        };
        record(value);
        if let Value::Instruction { instruction, .. } = &dfg[value]
            && let Instruction::Cast(input, _) = &dfg[*instruction]
        {
            record(*input);
        }
    }

    /// Whether `length > 0` is known.
    fn is_non_empty(&self, dfg: &DataFlowGraph, length: ValueId) -> bool {
        self.always_fails
            || self.non_zero.contains(&length)
            || self.known_value(dfg, length).is_some_and(|length| length > 0)
    }

    /// The constant value of `value`, or the constant a constraint pins it to.
    fn known_value(&self, dfg: &DataFlowGraph, value: ValueId) -> Option<u128> {
        constant_value(dfg, value).or_else(|| self.equal_to.get(&value).copied())
    }

    /// Whether `index < length` (or `index <= length` when `inclusive`) is known.
    fn index_in_bounds(
        &self,
        dfg: &DataFlowGraph,
        index: ValueId,
        length: ValueId,
        inclusive: bool,
    ) -> bool {
        if self.always_fails {
            return true;
        }
        let known = |value| self.known_value(dfg, value);
        // The exclusive upper bound on the index, when the length is known.
        let max_bound = known(length).map(|length| length + u128::from(inclusive));

        let index_value = known(index);
        if let (Some(index), Some(max_bound)) = (index_value, max_bound) {
            return index < max_bound;
        }
        if index_value == Some(0) && (inclusive || self.non_zero.contains(&length)) {
            return true;
        }

        let bound_is_enough = |bound: &ValueId| {
            *bound == length
                || (inclusive && is_increment_of(dfg, *bound, length))
                || matches!((constant_value(dfg, *bound), max_bound), (Some(bound), Some(max)) if bound <= max)
        };
        let checked_by_lt =
            self.less_than.get(&index).is_some_and(|bounds| bounds.iter().any(bound_is_enough));
        let checked_by_range_check = matches!(
            (self.bit_bounds.get(&index), max_bound),
            (Some(bits), Some(max)) if *bits < 128 && 1u128 << bits <= max
        );
        checked_by_lt || checked_by_range_check
    }
}

/// Whether `value` is `add base, 1` (or `add 1, base`).
fn is_increment_of(dfg: &DataFlowGraph, value: ValueId, base: ValueId) -> bool {
    let Value::Instruction { instruction, .. } = &dfg[value] else {
        return false;
    };
    let Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Add { .. } }) =
        &dfg[*instruction]
    else {
        return false;
    };
    (*lhs == base && constant_value(dfg, *rhs) == Some(1))
        || (*rhs == base && constant_value(dfg, *lhs) == Some(1))
}

fn constant_value(dfg: &DataFlowGraph, value: ValueId) -> Option<u128> {
    dfg.get_numeric_constant(value)?.try_into_u128()
}

#[cfg(test)]
mod tests {
    use crate::ssa::ssa_gen::Ssa;

    /// Parses `body` as the single block of a `main` taking `v0: u32` (the index),
    /// `v1: u32` (the length) and `v2: [Field]` (the vector).
    fn validate(body: &str) {
        let src = format!(
            "
            acir(inline) fn main f0 {{
              b0(v0: u32, v1: u32, v2: [Field]):
                {body}
                return
            }}
            "
        );
        let _ = Ssa::from_str(&src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_insert call in function f0 is not preceded by a bounds check"
    )]
    fn insert_without_check() {
        validate("v3, v4 = call vector_insert(v1, v2, v0, Field 9) -> (u32, [Field])");
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_without_check() {
        validate("v3, v4, v5 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)");
    }

    #[test]
    fn insert_checked_against_length_plus_one() {
        validate(
            "v3 = add v1, u32 1
             v4 = lt v0, v3
             constrain v4 == u1 1
             v5, v6 = call vector_insert(v1, v2, v0, Field 9) -> (u32, [Field])",
        );
    }

    #[test]
    fn remove_checked_against_length() {
        validate(
            "v3 = lt v0, v1
             constrain v3 == u1 1
             v4, v5, v6 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)",
        );
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_checked_against_length_plus_one() {
        validate(
            "v3 = add v1, u32 1
             v4 = lt v0, v3
             constrain v4 == u1 1
             v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)",
        );
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn check_after_the_call() {
        validate(
            "v4, v5, v6 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
             v3 = lt v0, v1
             constrain v3 == u1 1",
        );
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn check_in_a_preceding_block() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v3 = lt v0, v1
            constrain v3 == u1 1
            jmp b1()
          b1():
            v4, v5, v6 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_range_checked_against_power_of_two_length() {
        validate(
            "v3 = cast v0 as Field
             range_check v3 to 2 bits
             v4, v5, v6 = call vector_remove(u32 4, v2, v0) -> (u32, [Field], Field)",
        );
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_range_checked_to_too_many_bits() {
        validate(
            "v3 = cast v0 as Field
             range_check v3 to 3 bits
             v4, v5, v6 = call vector_remove(u32 4, v2, v0) -> (u32, [Field], Field)",
        );
    }

    #[test]
    fn remove_with_cast_index_constrained_to_zero() {
        validate(
            "v3 = cast v0 as Field
             constrain v3 == Field 0
             v4, v5, v6 = call vector_remove(u32 1, v2, v0) -> (u32, [Field], Field)",
        );
    }

    #[test]
    fn constant_index_in_bounds() {
        validate("v3, v4 = call vector_insert(u32 2, v2, u32 2, Field 9) -> (u32, [Field])");
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn constant_index_out_of_bounds() {
        validate("v3, v4, v5 = call vector_remove(u32 2, v2, u32 2) -> (u32, [Field], Field)");
    }

    #[test]
    fn constant_index_out_of_bounds_after_failing_constraint() {
        validate(
            "constrain u1 0 == u1 1
             v3, v4, v5 = call vector_remove(u32 2, v2, u32 2) -> (u32, [Field], Field)",
        );
    }

    #[test]
    fn insert_at_index_zero() {
        validate("v3, v4 = call vector_insert(v1, v2, u32 0, Field 9) -> (u32, [Field])");
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_at_index_zero_without_check() {
        validate("v3, v4, v5 = call vector_remove(v1, v2, u32 0) -> (u32, [Field], Field)");
    }

    #[test]
    fn remove_at_index_zero_with_length_checked_non_zero() {
        validate(
            "v3 = eq v1, u32 0
             constrain v3 == u1 0
             v4, v5, v6 = call vector_remove(v1, v2, u32 0) -> (u32, [Field], Field)",
        );
    }

    /// Parses `body` as the single block of a Brillig `main` taking `v0: u32` (the length) and
    /// `v1: [Field]` (the vector).
    fn validate_brillig(body: &str) {
        let src = format!(
            "
            brillig(inline) fn main f0 {{
              b0(v0: u32, v1: [Field]):
                {body}
                return
            }}
            "
        );
        let _ = Ssa::from_str(&src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_pop_back call in function f0 is not preceded by a bounds check"
    )]
    fn brillig_pop_back_without_check() {
        validate_brillig("v2, v3, v4 = call vector_pop_back(v0, v1) -> (u32, [Field], Field)");
    }

    #[test]
    #[should_panic(
        expected = "vector_pop_front call in function f0 is not preceded by a bounds check"
    )]
    fn brillig_pop_front_without_check() {
        validate_brillig("v2, v3, v4 = call vector_pop_front(v0, v1) -> (Field, u32, [Field])");
    }

    #[test]
    fn brillig_pop_back_with_length_checked_non_zero() {
        validate_brillig(
            "v2 = lt u32 0, v0
             constrain v2 == u1 1
             v3, v4, v5 = call vector_pop_back(v0, v1) -> (u32, [Field], Field)",
        );
    }

    #[test]
    fn brillig_pop_front_with_constant_non_zero_length() {
        validate_brillig("v2, v3, v4 = call vector_pop_front(u32 2, v1) -> (Field, u32, [Field])");
    }

    #[test]
    fn brillig_pop_back_of_empty_vector_after_failing_constraint() {
        validate_brillig(
            "constrain u1 0 == u1 1, \"Attempt to pop from an empty vector\"
             v2, v3, v4 = call vector_pop_back(u32 0, v1) -> (u32, [Field], Field)",
        );
    }

    #[test]
    #[should_panic(
        expected = "vector_pop_back call in function f0 is not preceded by a bounds check"
    )]
    fn brillig_pop_back_of_empty_vector() {
        validate_brillig("v2, v3, v4 = call vector_pop_back(u32 0, v1) -> (u32, [Field], Field)");
    }

    #[test]
    fn acir_pop_back_without_check() {
        validate("v3, v4, v5 = call vector_pop_back(v1, v2) -> (u32, [Field], Field)");
    }
}
