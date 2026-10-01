//! Validates that vector intrinsics which select an element by index (or which need a non-empty
//! vector) only run once that index (or length) is known to be in range.
//!
//! SSA generation emits a bounds check before each of these calls (see
//! `codegen_intrinsic_call_checks`), and the intrinsics themselves assume it held:
//!
//! | intrinsic                                      | requirement    |
//! |------------------------------------------------|----------------|
//! | `vector_insert(len, vector, index, ..)`        | `index <= len` |
//! | `vector_remove(len, vector, index)`            | `index < len`  |
//! | `vector_pop_back/front(len, vector)` (Brillig) | `0 < len`      |
//!
//! ACIR pops have no guard: ACIR generation handles popping from an empty vector itself.
//!
//! The check is not required to be syntactically present, because the instruction simplifier
//! removes or rewrites it when it can. A call is accepted when its requirement follows from
//! constants or from a dominating instruction of one of these shapes:
//!
//! - `constrain (lt a, b) == u1 1`, proving `a < b`. For `vector_insert`, `b` may be
//!   `add len, u32 1`.
//! - `range_check v to k bits`, proving `v < 2^k`, where `v` is the index or a cast of it.
//!   This is how ACIR checks an index against a power-of-two bound.
//! - `constrain v == c` for a constant `c`, which is what `lt v, u32 1` simplifies to. `v` may
//!   also be a cast of the index, which is what a range check to 0 bits simplifies to.
//! - `constrain (eq v, 0) == u1 0` (or `not` of it), which is what ACIR simplifies `lt 0, v` to.
//! - A constraint between two different constants, which always fails, so nothing after it runs.
//!
//! Optimization passes can prove the same facts in ways this rule does not recognise (for example
//! flattening predicates the constraint), so it only runs in the full validation mode.
use acvm::AcirField;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    cfg::ControlFlowGraph,
    dfg::DataFlowGraph,
    dom::{DominanceQueries, DominatorTree},
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, Intrinsic},
    post_order::PostOrder,
    value::{Value, ValueId},
};

/// Panics if a vector intrinsic in `function` is not guarded by a proof that its index or length
/// is in range. See the module documentation for the accepted shapes.
pub(super) fn validate_vector_bounds_guards(function: &Function) {
    let dfg = &function.dfg;
    let cfg = ControlFlowGraph::with_function(function);
    let post_order = PostOrder::with_cfg(&cfg);
    let dom = DominatorTree::with_cfg_and_post_order(&cfg, &post_order, DominanceQueries::Enabled);
    let blocks = post_order.into_vec_reverse();

    let has_guarded_call = blocks.iter().any(|block| {
        dfg[*block]
            .instructions()
            .iter()
            .any(|instruction| guarded_intrinsic(function, *instruction).is_some())
    });
    if !has_guarded_call {
        return;
    }

    let mut facts = Facts {
        dom,
        less_than_by_lhs: HashMap::default(),
        less_than_by_rhs: HashMap::default(),
        bit_bounds: HashMap::default(),
        equal_to: HashMap::default(),
        non_zero: HashMap::default(),
        always_failing: Vec::new(),
    };

    for block in blocks {
        for instruction in dfg[block].instructions() {
            match &dfg[*instruction] {
                Instruction::Constrain(lhs, rhs, _) => {
                    facts.record_constrain(dfg, *lhs, *rhs, block);
                }
                Instruction::RangeCheck { value, max_bit_size, .. } => {
                    facts.record_range_check(dfg, *value, *max_bit_size, block);
                }
                Instruction::Call { arguments, .. } => {
                    let Some(intrinsic) = guarded_intrinsic(function, *instruction) else {
                        continue;
                    };
                    let length = arguments[0];
                    let in_range = facts.unreachable(block)
                        || match intrinsic {
                            Intrinsic::VectorInsert => {
                                facts.index_in_range(dfg, arguments[2], length, block, true)
                            }
                            Intrinsic::VectorRemove => {
                                facts.index_in_range(dfg, arguments[2], length, block, false)
                            }
                            _ => facts.length_is_non_zero(dfg, length, block),
                        };
                    assert!(
                        in_range,
                        "{intrinsic} call in function {} is not preceded by a bounds check: {}",
                        function.id(),
                        requirement(intrinsic),
                    );
                }
                _ => (),
            }
        }
    }
}

/// The intrinsic called by `instruction`, if it is one whose index or length must be checked
/// beforehand.
fn guarded_intrinsic(function: &Function, instruction: InstructionId) -> Option<Intrinsic> {
    let Instruction::Call { func, .. } = &function.dfg[instruction] else {
        return None;
    };
    let Value::Intrinsic(intrinsic) = function.dfg[*func] else {
        return None;
    };
    match intrinsic {
        Intrinsic::VectorInsert | Intrinsic::VectorRemove => Some(intrinsic),
        Intrinsic::VectorPopBack | Intrinsic::VectorPopFront if function.runtime().is_brillig() => {
            Some(intrinsic)
        }
        _ => None,
    }
}

fn requirement(intrinsic: Intrinsic) -> &'static str {
    match intrinsic {
        Intrinsic::VectorInsert => "the index must be known to be at most the vector length",
        Intrinsic::VectorRemove => "the index must be known to be less than the vector length",
        _ => "the vector length must be known to be non-zero",
    }
}

/// Facts established by constraints and range checks, with the block each one holds from.
struct Facts {
    dom: DominatorTree,
    /// `lhs -> [(rhs, block)]`: `lhs < rhs` holds in blocks dominated by `block`.
    less_than_by_lhs: HashMap<ValueId, Vec<(ValueId, BasicBlockId)>>,
    /// `rhs -> [(lhs, block)]`: the same facts, keyed by the right-hand side.
    less_than_by_rhs: HashMap<ValueId, Vec<(ValueId, BasicBlockId)>>,
    /// `value -> [(bits, block)]`: `value < 2^bits`.
    bit_bounds: HashMap<ValueId, Vec<(u32, BasicBlockId)>>,
    /// `value -> [(constant, block)]`: `value == constant`.
    equal_to: HashMap<ValueId, Vec<(u128, BasicBlockId)>>,
    /// `value -> [((), block)]`: `value != 0`.
    non_zero: HashMap<ValueId, Vec<((), BasicBlockId)>>,
    /// Blocks containing a constraint that always fails.
    always_failing: Vec<((), BasicBlockId)>,
}

impl Facts {
    fn record_constrain(
        &mut self,
        dfg: &DataFlowGraph,
        lhs: ValueId,
        rhs: ValueId,
        block: BasicBlockId,
    ) {
        let (value, constant) = match (constant_u128(dfg, lhs), constant_u128(dfg, rhs)) {
            (None, Some(constant)) => (lhs, constant),
            (Some(constant), None) => (rhs, constant),
            (Some(lhs), Some(rhs)) => {
                if lhs != rhs {
                    self.always_failing.push(((), block));
                }
                return;
            }
            (None, None) => return,
        };
        self.record_equal(dfg, value, constant, block);
    }

    /// Records `value == constant`, along with what that implies for the operands of `value`.
    fn record_equal(
        &mut self,
        dfg: &DataFlowGraph,
        value: ValueId,
        constant: u128,
        block: BasicBlockId,
    ) {
        self.equal_to.entry(value).or_default().push((constant, block));
        if constant != 0 {
            self.non_zero.entry(value).or_default().push(((), block));
        }

        let Value::Instruction { instruction, .. } = &dfg[value] else {
            return;
        };
        match (&dfg[*instruction], constant) {
            (Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Lt }), 1) => {
                self.less_than_by_lhs.entry(*lhs).or_default().push((*rhs, block));
                self.less_than_by_rhs.entry(*rhs).or_default().push((*lhs, block));
            }
            (Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Eq }), 0) => {
                if constant_u128(dfg, *rhs) == Some(0) {
                    self.non_zero.entry(*lhs).or_default().push(((), block));
                } else if constant_u128(dfg, *lhs) == Some(0) {
                    self.non_zero.entry(*rhs).or_default().push(((), block));
                }
            }
            (Instruction::Not(input), 0 | 1) => {
                self.record_equal(dfg, *input, 1 - constant, block);
            }
            // A cast that does not fail preserves the value.
            (Instruction::Cast(input, _), _) => self.record_equal(dfg, *input, constant, block),
            _ => (),
        }
    }

    fn record_range_check(
        &mut self,
        dfg: &DataFlowGraph,
        value: ValueId,
        bits: u32,
        block: BasicBlockId,
    ) {
        self.bit_bounds.entry(value).or_default().push((bits, block));
        // ACIR range checks an index through a cast to `Field`; a cast that does not wrap
        // preserves the value, so the bound holds for the cast input as well.
        if let Value::Instruction { instruction, .. } = &dfg[value]
            && let Instruction::Cast(input, _) = &dfg[*instruction]
        {
            self.bit_bounds.entry(*input).or_default().push((bits, block));
        }
    }

    /// The facts from `facts` that hold in `block`: those recorded in a block dominating it.
    fn holds_at<T: Copy>(
        &self,
        facts: Option<&Vec<(T, BasicBlockId)>>,
        block: BasicBlockId,
    ) -> Vec<T> {
        let facts = facts.map(Vec::as_slice).unwrap_or_default();
        facts
            .iter()
            .filter(|(_, from)| self.dom.dominates(*from, block))
            .map(|(fact, _)| *fact)
            .collect()
    }

    /// Whether a constraint that always fails dominates `block`.
    fn unreachable(&self, block: BasicBlockId) -> bool {
        !self.holds_at(Some(&self.always_failing), block).is_empty()
    }

    /// The constant value of `value` at `block`, either because it is a constant or because a
    /// dominating constraint pins it to one.
    fn known_value(
        &self,
        dfg: &DataFlowGraph,
        value: ValueId,
        block: BasicBlockId,
    ) -> Option<u128> {
        constant_u128(dfg, value)
            .or_else(|| self.holds_at(self.equal_to.get(&value), block).first().copied())
    }

    /// Whether `index < length` (or `index <= length` when `inclusive`) is known at `block`.
    fn index_in_range(
        &self,
        dfg: &DataFlowGraph,
        index: ValueId,
        length: ValueId,
        block: BasicBlockId,
        inclusive: bool,
    ) -> bool {
        let length_value = self.known_value(dfg, length, block);
        // The largest exclusive upper bound on the index that keeps the call in range.
        let max_bound = length_value.map(|length| length + u128::from(inclusive));

        let index_value = self.known_value(dfg, index, block);
        if let (Some(index), Some(max_bound)) = (index_value, max_bound) {
            return index < max_bound;
        }
        // Index 0 is in range for an insert into any vector, and for a remove from a non-empty one.
        if index_value == Some(0) && (inclusive || self.length_is_non_zero(dfg, length, block)) {
            return true;
        }

        let bound_is_enough = |bound: ValueId| {
            bound == length
                || (inclusive && is_increment_of(dfg, bound, length))
                || max_bound.is_some_and(|max_bound| {
                    constant_u128(dfg, bound).is_some_and(|bound| bound <= max_bound)
                })
        };
        if self.holds_at(self.less_than_by_lhs.get(&index), block).into_iter().any(bound_is_enough)
        {
            return true;
        }

        max_bound.is_some_and(|max_bound| {
            self.holds_at(self.bit_bounds.get(&index), block)
                .into_iter()
                .any(|bits| bits < 128 && (1u128 << bits) <= max_bound)
        })
    }

    /// Whether `length > 0` is known at `block`.
    fn length_is_non_zero(
        &self,
        dfg: &DataFlowGraph,
        length: ValueId,
        block: BasicBlockId,
    ) -> bool {
        if let Some(length) = self.known_value(dfg, length, block) {
            return length > 0;
        }
        if !self.holds_at(self.non_zero.get(&length), block).is_empty() {
            return true;
        }
        // `c < length` for any constant `c` means `length` is non-zero.
        self.holds_at(self.less_than_by_rhs.get(&length), block)
            .into_iter()
            .any(|lhs| constant_u128(dfg, lhs).is_some())
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
    (*lhs == base && constant_u128(dfg, *rhs) == Some(1))
        || (*rhs == base && constant_u128(dfg, *lhs) == Some(1))
}

fn constant_u128(dfg: &DataFlowGraph, value: ValueId) -> Option<u128> {
    dfg.get_numeric_constant(value)?.try_into_u128()
}

#[cfg(test)]
mod tests {
    use crate::ssa::ssa_gen::Ssa;

    #[test]
    #[should_panic(
        expected = "vector_insert call in function f0 is not preceded by a bounds check"
    )]
    fn insert_with_unchecked_index() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = make_array [Field 1, Field 2] : [Field]
            v2, v3 = call vector_insert(u32 2, v1, v0, Field 9) -> (u32, [Field])
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_with_unchecked_index() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = make_array [Field 1, Field 2] : [Field]
            v2, v3, v4 = call vector_remove(u32 2, v1, v0) -> (u32, [Field], Field)
            return v2, v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn insert_checked_against_length_plus_one() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v4 = add v1, u32 1
            v5 = lt v0, v4
            constrain v5 == u1 1
            v7, v8 = call vector_insert(v1, v2, v0, Field 9) -> (u32, [Field])
            return v7
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_checked_against_length() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v3 = lt v0, v1
            constrain v3 == u1 1
            v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            return v5, v7
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_checked_against_length_plus_one() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v4 = add v1, u32 1
            v5 = lt v0, v4
            constrain v5 == u1 1
            v7, v8, v9 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            return v7, v9
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_range_checked_against_power_of_two_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = make_array [Field 1, Field 2, Field 3, Field 4] : [Field]
            v2 = cast v0 as Field
            range_check v2 to 2 bits
            v4, v5, v6 = call vector_remove(u32 4, v1, v0) -> (u32, [Field], Field)
            return v4, v6
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_range_checked_to_too_many_bits() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = make_array [Field 1, Field 2, Field 3, Field 4] : [Field]
            v2 = cast v0 as Field
            range_check v2 to 3 bits
            v4, v5, v6 = call vector_remove(u32 4, v1, v0) -> (u32, [Field], Field)
            return v4, v6
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_with_index_constrained_to_constant() {
        // `lt v0, u32 1` simplifies to `eq v0, u32 0`, and the constraint on it to this.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            constrain v0 == u32 0
            v3, v4, v5 = call vector_remove(u32 1, v1, v0) -> (u32, [Field], Field)
            return v3, v5
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn insert_with_constant_index_in_range() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field]):
            v3, v4 = call vector_insert(u32 2, v0, u32 2, Field 9) -> (u32, [Field])
            return v3
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_insert call in function f0 is not preceded by a bounds check"
    )]
    fn insert_with_constant_index_out_of_range() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field]):
            v3, v4 = call vector_insert(u32 2, v0, u32 3, Field 9) -> (u32, [Field])
            return v3
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_with_constant_index_equal_to_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field]):
            v3, v4, v5 = call vector_remove(u32 2, v0, u32 2) -> (u32, [Field], Field)
            return v3, v5
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn check_after_the_call() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            v3 = lt v0, v1
            constrain v3 == u1 1
            return v5, v7
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn check_in_a_block_that_does_not_dominate_the_call() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field], v3: u1):
            jmpif v3 then: b1(), else: b2()
          b1():
            v4 = lt v0, v1
            constrain v4 == u1 1
            jmp b2()
          b2():
            v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            return v5, v7
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn check_in_a_dominating_block() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field], v3: u1):
            v4 = lt v0, v1
            constrain v4 == u1 1
            jmpif v3 then: b1(), else: b2()
          b1():
            v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            jmp b2()
          b2():
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_pop_back call in function f0 is not preceded by a bounds check"
    )]
    fn brillig_pop_back_with_unchecked_length() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2, v3, v4 = call vector_pop_back(v0, v1) -> (u32, [Field], Field)
            return v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn brillig_pop_front_with_checked_length() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2 = lt u32 0, v0
            constrain v2 == u1 1
            v3, v4, v5 = call vector_pop_front(v0, v1) -> (Field, u32, [Field])
            return v3
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn acir_pop_back_with_unchecked_length() {
        // ACIR generation handles popping from an empty vector itself.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2, v3, v4 = call vector_pop_back(v0, v1) -> (u32, [Field], Field)
            return v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn insert_at_index_zero_with_unknown_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v3, v4 = call vector_insert(v0, v1, u32 0, Field 9) -> (u32, [Field])
            return v3
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "vector_remove call in function f0 is not preceded by a bounds check"
    )]
    fn remove_at_index_zero_with_unknown_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v3, v4, v5 = call vector_remove(v0, v1, u32 0) -> (u32, [Field], Field)
            return v3, v5
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_at_index_zero_with_checked_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2 = lt u32 0, v0
            constrain v2 == u1 1
            v3, v4, v5 = call vector_remove(v0, v1, u32 0) -> (u32, [Field], Field)
            return v3, v5
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn acir_remove_at_index_zero_with_length_checked_non_zero() {
        // ACIR simplifies `lt u32 0, v0` to `not (eq v0, u32 0)`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2 = eq v0, u32 0
            v3 = not v2
            constrain v2 == u1 0
            v4, v5, v6 = call vector_remove(v0, v1, u32 0) -> (u32, [Field], Field)
            return v4, v6
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_with_cast_index_constrained_to_zero() {
        // A range check to 0 bits on `cast v0 as Field` simplifies to this.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            v2 = cast v0 as Field
            constrain v2 == Field 0
            v3, v4, v5 = call vector_remove(u32 1, v1, v0) -> (u32, [Field], Field)
            return v3, v5
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn brillig_pop_back_after_failing_constraint() {
        // Popping from a vector known to be empty: the check folds to a constraint that always
        // fails, so the call never runs.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            constrain u1 0 == u1 1
            v2, v3, v4 = call vector_pop_back(u32 0, v0) -> (u32, [Field], Field)
            return v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn remove_with_index_pinned_to_zero_and_checked_against_length() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: [Field]):
            v3 = cast v0 as Field
            constrain v3 == Field 0
            v4 = lt v0, v1
            constrain v4 == u1 1
            v5, v6, v7 = call vector_remove(v1, v2, v0) -> (u32, [Field], Field)
            return v5, v7
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }
}
