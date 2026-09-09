//! Post-flattening validation pass that ensures predicated values escaping
//! their enable-side-effect definition are properly predicated.
//!
//! In ACIR, the result of a
//! [`requires_acir_gen_predicate`][crate::ssa::ir::instruction::Instruction::requires_acir_gen_predicate]
//! instruction depends on the active `enable_side_effects` predicate. For
//! side-effecting *arithmetic* (the overflow-checked `Binary` operations) the
//! result is multiplied by the predicate, so it is `0` whenever the predicate is
//! false. The other predicated instructions consume the predicate differently
//! (for example `ArrayGet` clamps its index and a `Call` forwards the predicate
//! to the callee), so their result is not necessarily `0` when the predicate is
//! false; this pass conservatively tracks all of them and still requires them to
//! be guarded before they can escape.
//!
//! Some optimization passes (such as `checked_to_unchecked`) can remove
//! *side-effect* checks and rewrite instructions into ones having
//! `requires_acir_gen_predicate` set to `false`. The rewritten instruction's
//! result is then no longer `0` when the predicate is false. This stays sound
//! only because flattening adds a predicate multiplication, so the escaping
//! value is still `0` when the predicate is false.
//!
//! This pass makes that an invariant and validates that no `requires_acir_gen_predicate`
//! value can escape its enclosing `enable_side_effects` without a predicate
//! multiplication (or an equivalent `IfElse` merge, which gates each branch by
//! its own condition).
//!
//! As a result, any optimization on `requires_acir_gen_predicate` done after
//! flattening is ensured to be sound.
//!
//! This is a post-flattening check, so each ACIR function is a single block. The
//! pass asserts this rather than assuming it, which makes the validation a simple
//! iteration over the instructions:
//! - mark `requires_acir_gen_predicate` instructions,
//! - mark also their uses,
//! - report a violation if used outside `enable_side_effects`,
//!   unless multiplied with the predicate.

use acvm::AcirField as _;
use noirc_errors::call_stack::CallStack;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::errors::{RtResult, RuntimeError};
use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

pub(crate) fn verify_side_effect_predicates(ssa: &Ssa) -> RtResult<()> {
    for function in ssa.functions.values() {
        verify_function(function)?;
    }
    Ok(())
}

fn verify_function(function: &Function) -> RtResult<()> {
    // Brillig functions do not have `enable_side_effects` instructions
    if function.runtime().is_brillig() {
        return Ok(());
    }

    // The single-instruction-iteration below only inspects the entry block, which
    // is only complete because flattening reduces every ACIR function to one block.
    // Assert the invariant instead of trusting it: it is a cheap check when it holds
    // and turns a future regression into a clear failure rather than silently
    // skipping the instructions of any other block.
    assert_eq!(
        function.reachable_blocks().len(),
        1,
        "ACIR function {} should be a single block after flattening",
        function.name()
    );

    let dfg = &function.dfg;
    let block = function.entry_block();

    // Values resulting from `requires_acir_gen_predicate` instructions, or
    // using such predicated values. They are mapped to their predicate.
    // These are the 'unsafe' values that we want to track for a non-predicated
    // use outside their enable-side-effect definition.
    let mut predicated_values: HashMap<ValueId, ValueId> = HashMap::default();

    // Predicates that hold in every satisfying assignment need no guard, see
    // `is_constrained_to_one`.
    let constrained = collect_constrained_booleans(dfg, block);

    // The active predicate: `None` if no predicate (or `1`).
    let mut current: Option<ValueId> = None;

    for instruction_id in dfg[block].instructions() {
        let instruction = &dfg[*instruction_id];

        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            current = (!is_one(function, *condition)).then_some(*condition);
            continue;
        }

        // Match instructions for
        // - using predicated operands
        // - using predicate operands outside enable-side-effect context
        // - predicate guards: predicate * value
        let mut guard_a_predicated_value = false;
        let mut use_a_predicated_value = None;
        let mut violation = None;
        instruction.for_each_value(|operand| {
            // If the instruction uses a `predicated_value`
            let Some(&p) = predicated_values.get(&operand) else {
                return;
            };
            if is_predicate_guard(dfg, instruction, operand, p) {
                guard_a_predicated_value = true;
            } else {
                // Propagate the predicate to the current instruction
                use_a_predicated_value.get_or_insert(p);
                if current.is_none() {
                    // The `predicated_value` operand is not used under a predicate,
                    // flag it as an error.
                    violation.get_or_insert(operand);
                }
            }
        });
        if let Some(operand) = violation {
            let call_stack = function.dfg.get_instruction_call_stack(*instruction_id);
            return Err(escape_error(function, operand, call_stack));
        }

        // `result_predicate` is the predicate of the current requires_acir_gen_predicate instruction,
        // or the predicate of the instruction's operand, if any.
        // When the instruction does not depend on a predicate, it is `None`
        let result_predicate = if guard_a_predicated_value {
            // A guard is safe
            None
        } else if use_a_predicated_value.is_some() {
            use_a_predicated_value
        } else if instruction.requires_acir_gen_predicate(dfg)
            && !is_div_or_mod_by_nonzero_constant(dfg, instruction)
        {
            current
        } else {
            None
        };
        if let Some(p) = result_predicate {
            // A predicate that is pinned to `1` is never false, so the results computed
            // under it are never the disabled-branch value and need no guard to escape.
            if !is_constrained_to_one(dfg, p, &constrained) {
                for result in dfg.instruction_results(*instruction_id) {
                    predicated_values.insert(*result, p);
                }
            }
        }
    }

    // Error on returning an unsafe value
    if let Some(TerminatorInstruction::Return { return_values, call_stack }) =
        dfg[block].terminator()
    {
        for value in return_values {
            if predicated_values.contains_key(value) {
                let call_stack = function.dfg.get_call_stack(*call_stack);
                return Err(escape_error(function, *value, call_stack));
            }
        }
    }

    Ok(())
}

/// The values that unconditional `constrain x == 0` and `constrain x == 1` instructions
/// pin to a boolean constant.
#[derive(Default)]
struct ConstrainedBooleans {
    to_zero: HashSet<ValueId>,
    to_one: HashSet<ValueId>,
}

impl ConstrainedBooleans {
    fn contains(&self, value: ValueId, target_is_one: bool) -> bool {
        let pinned = if target_is_one { &self.to_one } else { &self.to_zero };
        pinned.contains(&value)
    }
}

/// Collect the values that an unconditional `constrain x == 0` or `constrain x == 1` pins.
///
/// A flattened ACIR function is a single block, so every `Constrain` in it is enforced
/// on every execution. A value constrained here is therefore pinned in *every* satisfying
/// assignment, no matter where in the block the constraint sits — which is why this is
/// gathered up front rather than as the instructions are walked.
fn collect_constrained_booleans(dfg: &DataFlowGraph, block: BasicBlockId) -> ConstrainedBooleans {
    let mut constrained = ConstrainedBooleans::default();
    for instruction_id in dfg[block].instructions() {
        let Instruction::Constrain(lhs, rhs, _) = &dfg[*instruction_id] else {
            continue;
        };
        for (value, expected) in [(lhs, rhs), (rhs, lhs)] {
            let Some(constant) = dfg.get_numeric_constant(*expected) else {
                continue;
            };
            if constant.is_one() {
                constrained.to_one.insert(*value);
            } else if constant.is_zero() {
                constrained.to_zero.insert(*value);
            }
        }
    }
    constrained
}

/// Whether predicate `p` is `1` in every satisfying assignment, in which case a value
/// computed under it is never the disabled-branch value and may escape ungated.
///
/// This is what keeps the decomposition of a guarded assertion valid. `decompose_constrain`
/// rewrites a boolean `constrain (p * value) == 1` into `constrain p == 1` and
/// `constrain value == 1`, dropping the multiplication that guarded `value`. The two forms
/// admit exactly the same assignments (`p = 1` and `value = 1`), so the bare use of `value`
/// is not an over-constraint — but it is only faithful *because* `p` is pinned alongside it.
fn is_constrained_to_one(
    dfg: &DataFlowGraph,
    value: ValueId,
    constrained: &ConstrainedBooleans,
) -> bool {
    is_constrained_to(dfg, value, true, constrained)
}

/// Whether `value` is pinned to `1` (`target_is_one`) or to `0` in every satisfying
/// assignment.
///
/// The proof rarely names the value directly, because `decompose_constrain` rewrites a
/// constraint into constraints on the operands that produced it:
/// - a product is `1` exactly when both factors are, and `0` as soon as either is, so
///   `p = p_outer * p_inner` — the conjunction flattening builds for a nested branch — is
///   proven by `constrain p_outer == 1` plus `constrain p_inner == 1`;
/// - a negation inverts the target: flattening gates an else branch by `not condition`, and
///   `constrain (not condition) == 1` is rewritten to `constrain condition == 0`, so the
///   proof names the source of the negation rather than the predicate itself. This arm is
///   restricted to booleans, where `not` is `1 - x`; on a wider integer it is a bitwise
///   complement and `not x == 1` would not follow from `x == 0`.
///
/// Casts are followed towards the source: a boolean constant casts to itself in any numeric
/// type, so a pinned source pins every cast of it. The converse is not assumed, since
/// narrowing casts are not injective.
fn is_constrained_to(
    dfg: &DataFlowGraph,
    value: ValueId,
    target_is_one: bool,
    constrained: &ConstrainedBooleans,
) -> bool {
    if constrained.contains(value, target_is_one) {
        return true;
    }
    let value = strip_casts(dfg, value);
    if constrained.contains(value, target_is_one) {
        return true;
    }
    if let Some(constant) = dfg.get_numeric_constant(value) {
        return if target_is_one { constant.is_one() } else { constant.is_zero() };
    }
    let Value::Instruction { instruction, .. } = &dfg[value] else {
        return false;
    };
    match &dfg[*instruction] {
        Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Mul { .. } }) => {
            if target_is_one {
                is_constrained_to(dfg, *lhs, true, constrained)
                    && is_constrained_to(dfg, *rhs, true, constrained)
            } else {
                is_constrained_to(dfg, *lhs, false, constrained)
                    || is_constrained_to(dfg, *rhs, false, constrained)
            }
        }
        Instruction::Not(source) if *dfg.type_of_value(*source) == Type::bool() => {
            is_constrained_to(dfg, *source, !target_is_one, constrained)
        }
        _ => false,
    }
}

/// True if `instruction` re-applies predicate `p` to `operand`, zeroing it
/// whenever `p` is false: `mul p, operand` (either order), or a branch of an
/// `IfElse` whose condition is `p` and whose value is `operand`. A merge gates
/// each branch by its own condition (`then_condition * then_value +
/// else_condition * else_value`), so both branches are checked.
fn is_predicate_guard(
    dfg: &DataFlowGraph,
    instruction: &Instruction,
    operand: ValueId,
    p: ValueId,
) -> bool {
    match instruction {
        Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Mul { .. } }) => {
            (*lhs == operand && is_predicate(dfg, *rhs, p))
                || (*rhs == operand && is_predicate(dfg, *lhs, p))
        }
        Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
            (*then_value == operand && is_predicate(dfg, *then_condition, p))
                || (*else_value == operand && is_predicate(dfg, *else_condition, p))
        }
        _ => false,
    }
}

/// Whether multiplying by `value` re-applies predicate `p`, i.e. `value` is `0`
/// whenever `p` is `0`.
///
/// Predicates are `u1` conditions; casting one to a wider type preserves its
/// zero/non-zero-ness, so casts on either side are ignored when comparing. This
/// holds when `value` is `p` itself, but also when:
/// - `value` is a product `p * r` (in any nesting or operand order): such a
///   product is `0` whenever `p` is `0`, so `value * operand` is still `0` on the
///   disabled branch. Flattening gates a nested branch by the *conjunction* of all
///   enclosing conditions, so a value tainted by an outer predicate `p` is
///   re-guarded by an inner predicate of the form `p * inner_condition`.
/// - `value` and `p` are different casts of the same source: the signed-division
///   expansion, for example, derives both the branch predicate and the merge
///   multiplier as separate casts of one sign value.
/// - `value` is an `IfElse` merge `then_condition * then_value + else_condition *
///   else_value`: the sum is `0` whenever `p` is `0` exactly when each product term
///   is, and a product term is `0` whenever either of its factors is.
fn is_predicate(dfg: &DataFlowGraph, value: ValueId, p: ValueId) -> bool {
    let value = strip_casts(dfg, value);
    let p = strip_casts(dfg, p);
    if value == p {
        return true;
    }
    if let Value::Instruction { instruction, .. } = &dfg[value] {
        match &dfg[*instruction] {
            Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Mul { .. } }) => {
                return is_predicate(dfg, *lhs, p) || is_predicate(dfg, *rhs, p);
            }
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                return (is_predicate(dfg, *then_condition, p)
                    || is_predicate(dfg, *then_value, p))
                    && (is_predicate(dfg, *else_condition, p)
                        || is_predicate(dfg, *else_value, p));
            }
            _ => {}
        }
    }
    false
}

/// Follow a chain of `cast` instructions to the underlying source value.
fn strip_casts(dfg: &DataFlowGraph, mut value: ValueId) -> ValueId {
    while let Value::Instruction { instruction, .. } = &dfg[value] {
        if let Instruction::Cast(src, _) = &dfg[*instruction] {
            value = *src;
        } else {
            break;
        }
    }
    value
}

fn is_one(function: &Function, value: ValueId) -> bool {
    function.dfg.get_numeric_constant(value).is_some_and(|c| c.is_one())
}

fn is_div_or_mod_by_nonzero_constant(dfg: &DataFlowGraph, instruction: &Instruction) -> bool {
    let Instruction::Binary(Binary { rhs, operator: BinaryOp::Div | BinaryOp::Mod, .. }) =
        instruction
    else {
        return false;
    };
    dfg.get_numeric_constant(*rhs).is_some_and(|c| !c.is_zero())
}

fn escape_error(function: &Function, operand: ValueId, call_stack: CallStack) -> RuntimeError {
    let message =
        format!("Value {operand} escapes `enable_side_effects` in function {}", function.name());

    RuntimeError::SsaValidationError { message, call_stack }
}

#[cfg(test)]
mod tests {
    use super::verify_side_effect_predicates;
    use crate::ssa::interpreter::value::Value;
    use crate::ssa::ssa_gen::Ssa;

    #[test]
    fn ifelse_on_the_multiplier_side_is_recognized() {
        // Here the `IfElse` sits on the *predicate side*: it is the multiplier of a
        // `mul` guard, not the merge of the tainted value. `is_predicate`'s `IfElse`
        // arm sees that each branch of `if v1 then v0 else v0` is `0` whenever `v0`
        // is, so the merge is `0` whenever `v0` is, and the guard is recognized.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u16):
            enable_side_effects v0
            v3 = cast v2 as u32
            v4 = add v3, u32 1
            enable_side_effects u1 1
            v5 = not v1
            v6 = if v1 then v0 else (if v5) v0
            v7 = cast v6 as u32
            v8 = mul v7, v4
            return v8
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());

        // `v6 = v0`, so with `v0 = 0` the escaping value really is `0`.
        let disabled =
            ssa.interpret(vec![Value::bool(false), Value::bool(true), Value::u16(7)]).unwrap();
        assert_eq!(disabled, vec![Value::u32(0)]);
        let enabled =
            ssa.interpret(vec![Value::bool(true), Value::bool(true), Value::u16(7)]).unwrap();
        assert_eq!(enabled, vec![Value::u32(8)]);
    }

    #[test]
    fn rejects_ungated_escape_to_return() {
        // A checked add under `enable_side_effects v0`
        // is returned directly, so its disabled-branch zeroing is observable.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u16):
            enable_side_effects v0
            v2 = cast v1 as u32
            v3 = add v2, u32 1
            enable_side_effects u1 1
            return v3
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_err());
    }

    #[test]
    fn accepts_gated_escape() {
        // The flattened-frontend shape: the value leaves the region only through
        // `mul (cast v0), v3`, so it is `0` whenever `v0` is false.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u16):
            enable_side_effects v0
            v2 = cast v1 as u32
            v3 = add v2, u32 1
            enable_side_effects u1 1
            v4 = cast v0 as u32
            v5 = mul v4, v3
            return v5
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }

    #[test]
    fn accepts_non_side_effecting_op_under_predicate() {
        // `not`/`cast` are not zeroed at ACIR-gen (`requires_acir_gen_predicate`
        // is false), so a `not v0` defined under `enable_side_effects v0` is
        // unconditional and may escape ungated. This is the `regression_11268`
        // shape that an over-eager "anything under a predicate is tainted" check
        // would wrongly reject.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = not v0
            enable_side_effects u1 1
            v2 = cast v1 as Field
            return v2
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }

    #[test]
    fn rejects_taint_propagated_through_pure_op() {
        // The checked add is zeroed under `v0`; casting its result carries that
        // taint, so returning the cast ungated is still unsound.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u16):
            enable_side_effects v0
            v2 = cast v1 as u32
            v3 = add v2, u32 1
            v4 = cast v3 as Field
            enable_side_effects u1 1
            return v4
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_err());
    }

    #[test]
    fn accepts_div_by_nonzero_constant_escape() {
        // `div`/`mod` report `requires_acir_gen_predicate` so ACIR gen can guard against
        // division by zero. A division by a non-zero *constant* can never divide by zero:
        // `euclidean_division_var` returns the true quotient regardless of the predicate,
        // so the result is not predicated and may escape its region ungated. This is the
        // `0xc6ecd1e900100000` fuzzer shape, where a signed comparison (`cast (div x, 2^63)
        // as u1`) escaped into a `constrain`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            enable_side_effects v0
            v2 = div v1, u32 2
            enable_side_effects u1 1
            return v2
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }

    #[test]
    fn rejects_div_by_non_constant_escape() {
        // A division by a *runtime* divisor keeps requiring a predicate, so its result
        // stays tracked and an ungated escape is still rejected. This pins the
        // `accepts_div_by_nonzero_constant_escape` relaxation to the constant-divisor case.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32, v2: u32):
            enable_side_effects v0
            v3 = div v1, v2
            enable_side_effects u1 1
            return v3
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_err());
    }

    /// Escapes whose predicate the enclosing block proves is `1` in every satisfying
    /// assignment, and the near-misses that must still be rejected.
    ///
    /// Flattening guards a predicated value by multiplying it with its predicate, and this
    /// pass requires that guard before the value may leave its `enable_side_effects` region.
    /// `decompose_constrain` then rewrites a guarded assertion into constraints on the
    /// operands that produced it: the multiplication is dropped, but the predicate is pinned
    /// by a constraint of its own in its place. Recognizing those pinned predicates is what
    /// keeps the rewrite from looking like an unguarded escape, and every accepting case
    /// below is one shape the rewrite can produce.
    mod predicate_proofs {
        use test_case::test_case;

        use super::super::verify_side_effect_predicates;
        use crate::ssa::ssa_gen::Ssa;

        // Each constant is the body of a single-block ACIR `main` in which a checked `add` — a
        // `requires_acir_gen_predicate` instruction — is computed under a predicate and then
        // escapes into a `constrain` once the region has closed. The escaping value and the
        // escape itself are identical throughout; only the predicate's shape and the
        // constraints that pin it differ, so a case flipping means the *proof* was
        // (mis)recognized and nothing else.

        /// `constrain (p * value) == 1` on booleans is decomposed into `constrain p == 1` plus
        /// `constrain value == 1`, which strips the multiplication that guarded `value`. The
        /// rewrite is faithful precisely because `p` is pinned alongside it.
        const PREDICATE_PINNED_TO_ONE: &str = "
          b0(v0: u1, v1: u32):
            enable_side_effects v0
            v2 = add v1, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 1
            constrain v2 == u32 8
            return
        ";

        /// For a nested branch the predicate is the conjunction `v0 * v1`, and the decomposition
        /// recurses through the product to constrain the individual factors. The conjunction is
        /// `1` exactly when each factor is.
        const CONJOINED_PREDICATE_FACTORS_PINNED_TO_ONE: &str = "
          b0(v0: u1, v1: u1, v2: u32):
            enable_side_effects v0
            v3 = unchecked_mul v0, v1
            enable_side_effects v3
            v4 = add v2, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 1
            constrain v1 == u1 1
            constrain v4 == u32 8
            return
        ";

        /// Flattening gates an else branch by `not condition`, and `decompose_constrain` rewrites
        /// `constrain (not v0) == 1` into `constrain v0 == 0`, so the proof names the *source* of
        /// the negation rather than the predicate. `not v0` is still `1` everywhere.
        const NEGATED_PREDICATE_SOURCE_PINNED_TO_ZERO: &str = "
          b0(v0: u1, v1: u32):
            v2 = not v0
            enable_side_effects v2
            v3 = add v1, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 0
            constrain v3 == u32 8
            return
        ";

        /// A nested else branch negates a conjunction. `decompose_constrain` has no rule for
        /// `constrain (mul _ _) == 0`, so the proof names the product itself and the negation
        /// still has to be seen through.
        const NEGATED_CONJOINED_PREDICATE_PINNED_TO_ZERO: &str = "
          b0(v0: u1, v1: u1, v2: u32):
            v3 = unchecked_mul v0, v1
            v4 = not v3
            enable_side_effects v4
            v5 = add v2, u32 1
            enable_side_effects u1 1
            constrain v3 == u1 0
            constrain v5 == u32 8
            return
        ";

        /// Only `v0` is pinned, so the conjunction `v0 * v1` can still be `0` and the escaping
        /// value can still be the disabled-branch zero.
        const CONJOINED_PREDICATE_WITH_ONE_FACTOR_PINNED: &str = "
          b0(v0: u1, v1: u1, v2: u32):
            enable_side_effects v0
            v3 = unchecked_mul v0, v1
            enable_side_effects v3
            v4 = add v2, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 1
            constrain v4 == u32 8
            return
        ";

        /// Only a constraint pinning the predicate to `1` proves the enabled branch is taken;
        /// pinning it to `0` proves the opposite.
        const PREDICATE_PINNED_TO_ZERO: &str = "
          b0(v0: u1, v1: u32):
            enable_side_effects v0
            v2 = add v1, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 0
            constrain v2 == u32 8
            return
        ";

        /// `constrain v0 == 1` proves `not v0` is `0`, i.e. the guarded branch never ran. That is
        /// the opposite of a proof — the negated counterpart of [`PREDICATE_PINNED_TO_ZERO`].
        const NEGATED_PREDICATE_SOURCE_PINNED_TO_ONE: &str = "
          b0(v0: u1, v1: u32):
            v2 = not v0
            enable_side_effects v2
            v3 = add v1, u32 1
            enable_side_effects u1 1
            constrain v0 == u1 1
            constrain v3 == u32 8
            return
        ";

        /// Without any constraint on `v0`, `not v0` can be `0` and the escaping value can still
        /// be the disabled-branch zero.
        const NEGATED_PREDICATE_UNPINNED: &str = "
          b0(v0: u1, v1: u32):
            v2 = not v0
            enable_side_effects v2
            v3 = add v1, u32 1
            enable_side_effects u1 1
            constrain v3 == u32 8
            return
        ";

        fn main_with_body(body: &str) -> Ssa {
            Ssa::from_str_no_validation(&format!("acir(inline) fn main f0 {{{body}}}")).unwrap()
        }

        /// The block pins the active predicate to `1`, so the escaping value is never the
        /// disabled-branch value and needs no guard.
        #[test_case(PREDICATE_PINNED_TO_ONE; "predicate pinned to one")]
        #[test_case(CONJOINED_PREDICATE_FACTORS_PINNED_TO_ONE; "conjoined predicate factors pinned to one")]
        #[test_case(NEGATED_PREDICATE_SOURCE_PINNED_TO_ZERO; "negated predicate source pinned to zero")]
        #[test_case(NEGATED_CONJOINED_PREDICATE_PINNED_TO_ZERO; "negated conjoined predicate pinned to zero")]
        fn accepts_escape_when_predicate_is_proven(body: &str) {
            let ssa = main_with_body(body);

            if let Err(error) = verify_side_effect_predicates(&ssa) {
                panic!("expected the escape to be accepted, but validation rejected it: {error:?}");
            }
        }

        /// Each of these is a near-miss of an accepting case: the predicate can still be `0`, so
        /// the escaping value can still be the disabled-branch value and the guard is required.
        /// They pin the relaxation, in that widening the recognizer flips one of them.
        #[test_case(CONJOINED_PREDICATE_WITH_ONE_FACTOR_PINNED; "conjoined predicate with only one factor pinned")]
        #[test_case(PREDICATE_PINNED_TO_ZERO; "predicate pinned to zero")]
        #[test_case(NEGATED_PREDICATE_SOURCE_PINNED_TO_ONE; "negated predicate source pinned to one")]
        #[test_case(NEGATED_PREDICATE_UNPINNED; "negated predicate not pinned at all")]
        fn rejects_escape_when_predicate_is_unproven(body: &str) {
            let ssa = main_with_body(body);

            assert!(
                verify_side_effect_predicates(&ssa).is_err(),
                "expected the escape to be rejected, but validation accepted it"
            );
        }

        #[test]
        fn accepts_return_escape_when_predicate_is_proven() {
            // The cases above all escape through a `constrain`. A proven predicate also clears
            // the separate check on the block's `return` terminator.
            let ssa = main_with_body(
                "
              b0(v0: u1, v1: u32):
                enable_side_effects v0
                v2 = add v1, u32 1
                enable_side_effects u1 1
                constrain v0 == u1 1
                return v2
            ",
            );

            assert!(verify_side_effect_predicates(&ssa).is_ok());
        }
    }

    #[test]
    fn accepts_arithmetic_outside_any_region() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u16, v1: u16):
            v2 = cast v0 as u32
            v3 = cast v1 as u32
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }

    #[test]
    fn accepts_guard_by_conjoined_sub_predicate() {
        // A checked add tainted by `v0` is re-guarded by `v0 * v1`, the
        // conjunction flattening builds for a nested branch. `v0 * v1` is `0`
        // whenever `v0` is `0`, so the guard zeroes the value on the disabled
        // branch and the escape is spurious.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u16):
            enable_side_effects v0
            v3 = cast v2 as u32
            v4 = add v3, u32 1
            v5 = unchecked_mul v0, v1
            v6 = cast v5 as u32
            v7 = mul v6, v4
            enable_side_effects u1 1
            return v7
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }

    #[test]
    fn accepts_guard_by_other_cast_of_predicate_source() {
        // The branch predicate (`cast v0 as u1`) and the merge multiplier
        // (`cast v0 as u32`) are different casts of the same source `v0`, as the
        // signed-division expansion produces. Casts preserve zero-ness, so the
        // multiplier still zeroes the tainted value when the predicate is `0`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u16):
            v2 = cast v0 as u1
            enable_side_effects v2
            v3 = cast v1 as u32
            v4 = add v3, u32 1
            enable_side_effects u1 1
            v5 = cast v0 as u32
            v6 = mul v5, v4
            return v6
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        assert!(verify_side_effect_predicates(&ssa).is_ok());
    }
}
