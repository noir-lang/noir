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
use rustc_hash::FxHashMap as HashMap;

use crate::errors::{RtResult, RuntimeError};
use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
        value::ValueId,
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
            for result in dfg.instruction_results(*instruction_id) {
                predicated_values.insert(*result, p);
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

/// True if `instruction` re-applies predicate `p` to `operand`, zeroing it
/// whenever `p` is false: `mul p, operand` (either order), or a branch of an
/// `IfElse` whose condition is `p` and whose value is `operand`. A merge gates
/// each branch by its own condition (`then_condition * then_value +
/// else_condition * else_value`), so both branches are checked.
fn is_predicate_guard(
    dfg: &crate::ssa::ir::dfg::DataFlowGraph,
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

/// Whether `value` is the predicate `p`, possibly through `cast`s.
fn is_predicate(dfg: &crate::ssa::ir::dfg::DataFlowGraph, mut value: ValueId, p: ValueId) -> bool {
    loop {
        if value == p {
            return true;
        }
        match &dfg[value] {
            crate::ssa::ir::value::Value::Instruction { instruction, .. } => {
                if let Instruction::Cast(src, _) = &dfg[*instruction] {
                    value = *src;
                    continue;
                }
                return false;
            }
            _ => return false,
        }
    }
}

fn is_one(function: &Function, value: ValueId) -> bool {
    function.dfg.get_numeric_constant(value).is_some_and(|c| c.is_one())
}

fn is_div_or_mod_by_nonzero_constant(
    dfg: &crate::ssa::ir::dfg::DataFlowGraph,
    instruction: &Instruction,
) -> bool {
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
    use crate::ssa::ssa_gen::Ssa;

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
}
