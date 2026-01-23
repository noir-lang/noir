use crate::ssa::{
    ir::{
        dfg::simplify::SimplifyResult,
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, InstructionId,
            binary::{BinaryEvaluationResult, eval_constant_binary_op},
        },
        integer::IntegerConstant,
        value::ValueId,
    },
    opt::loop_invariant::BlockContext,
};
use noirc_errors::call_stack::CallStackId;

use super::{LoopContext, LoopInvariantContext};

impl LoopInvariantContext<'_> {
    /// Checks whether a binary operation can be evaluated using the bounds of a given loop induction variables.
    ///
    /// If it cannot be evaluated, it means that we either have a dynamic loop bound or
    /// that the operation can potentially overflow during a given loop iteration.
    pub(super) fn can_evaluate_binary_op(
        &self,
        loop_context: &LoopContext,
        binary: &Binary,
    ) -> bool {
        match binary.operator {
            BinaryOp::Div | BinaryOp::Mod => {
                // Division can be evaluated if we ensure that the divisor cannot be zero
                let Some((left, value, lower, upper)) =
                    self.match_induction_and_constant(loop_context, &binary.lhs, &binary.rhs, true)
                else {
                    // Not a constant vs non-constant case, we cannot evaluate it.
                    return false;
                };

                if left {
                    // If the induction variable is on the LHS, we're dividing with a constant.
                    if !value.is_zero() && !value.is_minus_one() {
                        return true;
                    }
                } else {
                    // Otherwise we are dividing a constant with the induction variable, and we have to check whether
                    // at any point in the loop the induction variable can be zero.
                    let can_be_zero = lower.is_negative() && !upper.is_negative()
                        || lower.is_zero()
                        || upper.is_zero();

                    if !can_be_zero {
                        return true;
                    }
                }

                false
            }
            // An unchecked operation cannot overflow, so it can be safely evaluated.
            // Some checked operations can be safely evaluated, depending on the loop bounds, but in that case,
            // they would have been already converted to unchecked operation in `simplify_induction_variable_in_binary()`.
            // These are all handled by `has_side_effects`, and are redundant with `can_be_hoisted`.
            _ => !binary.has_side_effects(&self.inserter.function.dfg),
        }
    }

    /// Some instructions can take advantage of that our induction variable has a fixed minimum/maximum,
    /// For instance operations can be transformed from a checked operation to an unchecked operation.
    ///
    /// Checked operations require more bytecode and thus we aim to minimize their usage wherever possible.
    ///
    /// For example, if one side of an add/mul operation is a constant and the other is an induction variable
    /// with a known upper bound, we know whether that binary operation will ever overflow.
    /// If we determine that an overflow is not possible we can convert the checked operation to unchecked.
    ///
    /// The function returns `false` if the instruction must be added to the block, which involves further simplification.
    pub(super) fn simplify_from_loop_bounds(
        &mut self,
        loop_context: &LoopContext,
        block_context: &BlockContext,
        instruction_id: InstructionId,
    ) -> bool {
        // Simplify the instruction and update it in the DFG.
        match self.simplify_induction_variable(loop_context, block_context, instruction_id) {
            SimplifyResult::SimplifiedTo(id) => {
                let [result] = self.inserter.function.dfg.instruction_result(instruction_id);
                self.inserter.map_value(result, id);
                true
            }
            SimplifyResult::SimplifiedToInstruction(instruction) => {
                self.inserter.function.dfg[instruction_id] = instruction;
                false
            }
            SimplifyResult::None => false,
            _ => unreachable!(
                "ICE - loop bounds simplification should only simplify to a value or an instruction"
            ),
        }
    }

    /// Replace 'assert(invariant != induction)' with assert((invariant < min(induction) || (invariant > max(induction)))
    /// For this simplification to be valid, we need to ensure that the induction variable takes all the values from min(induction) up to max(induction)
    /// This means that the assert must be executed at each loop iteration, and that the loop processes all the iteration space.
    /// This is ensured via control dependence and the check for break patterns, before calling this function.
    fn simplify_not_equal_constraint(
        &mut self,
        loop_context: &LoopContext,
        lhs: &ValueId,
        rhs: &ValueId,
        err: &Option<ConstrainError>,
        call_stack: CallStackId,
    ) -> SimplifyResult {
        let (invariant, min, max) = match self.match_induction_and_invariant(loop_context, lhs, rhs)
        {
            Some((true, min, max)) => (rhs, min, max),
            Some((false, min, max)) => (lhs, min, max),
            _ => return SimplifyResult::None,
        };

        let mut insert_binary_to_preheader = |lhs, rhs, operator| {
            let binary = Instruction::Binary(Binary { lhs, rhs, operator });
            let results = self
                .inserter
                .function
                .dfg
                .insert_instruction_and_results(binary, loop_context.pre_header(), None, call_stack)
                .results();
            assert!(results.len() == 1);
            results[0]
        };
        // The comparisons can be safely hoisted to the pre-header because they are loop invariant and control independent
        let check_min = insert_binary_to_preheader(*invariant, min, BinaryOp::Lt);
        let check_max = insert_binary_to_preheader(max, *invariant, BinaryOp::Lt);
        let check_bounds = insert_binary_to_preheader(check_min, check_max, BinaryOp::Or);

        SimplifyResult::SimplifiedToInstruction(Instruction::Constrain(
            check_bounds,
            self.true_value,
            err.clone(),
        ))
    }

    /// If the inputs are an induction and loop invariant variables, it returns
    /// the maximum and minimum values of the induction variable, based on the loop bounds,
    /// and a boolean indicating if the induction variable is on the lhs or rhs (true for lhs)
    fn match_induction_and_invariant(
        &mut self,
        loop_context: &LoopContext,
        lhs: &ValueId,
        rhs: &ValueId,
    ) -> Option<(bool, ValueId, ValueId)> {
        let (is_left, lower, upper) = match (
            loop_context.get_current_induction_variable_bounds(*lhs),
            loop_context.get_current_induction_variable_bounds(*rhs),
        ) {
            (_, Some((lower, upper))) => Some((false, lower, upper)),
            (Some((lower, upper)), _) => Some((true, lower, upper)),
            _ => None,
        }?;

        let (upper_field, upper_type) = upper.dec().into_numeric_constant();
        let (lower_field, lower_type) = lower.into_numeric_constant();

        let min_iter = self.inserter.function.dfg.make_constant(lower_field, lower_type);
        let max_iter = self.inserter.function.dfg.make_constant(upper_field, upper_type);
        if (is_left && loop_context.is_loop_invariant(rhs))
            || (!is_left && loop_context.is_loop_invariant(lhs))
        {
            return Some((is_left, min_iter, max_iter));
        }
        None
    }

    /// Simplify certain instructions using the lower/upper bounds of induction variables
    fn simplify_induction_variable(
        &mut self,
        loop_context: &LoopContext,
        block_context: &BlockContext,
        instruction_id: InstructionId,
    ) -> SimplifyResult {
        let (instruction, call_stack) = self.inserter.map_instruction(instruction_id);
        match &instruction {
            Instruction::Binary(binary) => self.simplify_induction_variable_in_binary(
                loop_context,
                binary,
                block_context.is_header,
            ),
            Instruction::ConstrainNotEqual(x, y, err) => {
                // Ensure the loop is fully executed
                if loop_context.no_break
                    && block_context.can_simplify_control_dependent_instruction()
                {
                    assert!(block_context.does_execute, "executing a non executable loop");
                    self.simplify_not_equal_constraint(loop_context, x, y, err, call_stack)
                } else {
                    SimplifyResult::None
                }
            }
            _ => SimplifyResult::None,
        }
    }

    /// If the inputs are an induction variable and a constant, it returns
    /// the constant value, the maximum and minimum values of the induction variable, based on the loop bounds,
    /// and a boolean indicating if the induction variable is on the lhs or rhs (true for lhs)
    /// if `only_outer_induction`, we only consider outer induction variables, else we also consider the induction variables from the current loop.
    fn match_induction_and_constant(
        &self,
        loop_context: &LoopContext,
        lhs: &ValueId,
        rhs: &ValueId,
        only_outer_induction: bool,
    ) -> Option<(bool, IntegerConstant, IntegerConstant, IntegerConstant)> {
        let lhs_const = self.inserter.function.dfg.get_integer_constant(*lhs);
        let rhs_const = self.inserter.function.dfg.get_integer_constant(*rhs);
        match (
            lhs_const,
            rhs_const,
            loop_context
                .get_current_induction_variable_bounds(*lhs)
                .filter(|_| !only_outer_induction)
                .or(self.outer_induction_variables.get(lhs).copied()),
            loop_context
                .get_current_induction_variable_bounds(*rhs)
                .filter(|_| !only_outer_induction)
                .or(self.outer_induction_variables.get(rhs).copied()),
        ) {
            // LHS is a constant, RHS is the induction variable with a known lower and upper bound.
            (Some(lhs), None, None, Some((lower_bound, upper_bound))) => {
                Some((false, lhs, lower_bound, upper_bound))
            }
            // RHS is a constant, LHS is the induction variable with a known lower an upper bound
            (None, Some(rhs), Some((lower_bound, upper_bound)), None) => {
                Some((true, rhs, lower_bound, upper_bound))
            }
            _ => None,
        }
    }

    /// Given a constant `c` and an induction variable `i`:
    /// - Replace comparisons `i < c` by true if `max(i) < c`, and false if `min(i) >= c`
    /// - Replace comparisons `c < i` by true if `min(i) > c`, and false if `max(i) <= c`
    /// - Replace equalities `i == c` by false if `min(i) > c or max(i) < c`
    /// - Replace checked operations with unchecked version if the induction variable bounds prove that the operation will not overflow
    ///
    /// `header` indicates if we are in the loop header where loop bounds do not apply yet
    fn simplify_induction_variable_in_binary(
        &mut self,
        loop_context: &LoopContext,
        binary: &Binary,
        is_header: bool,
    ) -> SimplifyResult {
        // Checks the operands are an induction variable and a constant
        // Note that here we allow all_induction_variables
        let operand_type = self.inserter.function.dfg.type_of_value(binary.lhs).unwrap_numeric();

        let Some((is_induction_var_lhs, constant, lower_bound, upper_bound)) =
            self.match_induction_and_constant(loop_context, &binary.lhs, &binary.rhs, is_header)
        else {
            return SimplifyResult::None;
        };

        // Handle arithmetic operations:
        // Check if we can simplify either `lower op const` or `const op upper` into an unchecked version of the operation.
        if let Some((lhs, rhs)) = match binary.operator {
            BinaryOp::Add { unchecked }
            | BinaryOp::Sub { unchecked }
            | BinaryOp::Mul { unchecked }
                if unchecked =>
            {
                // Already unchecked, no need to simplify.
                return SimplifyResult::None;
            }
            BinaryOp::Sub { .. } => {
                if is_induction_var_lhs {
                    // `i - const` won't overflow if the lowest `i` doesn't.
                    Some((lower_bound, constant))
                } else {
                    // `const - i` won't overflow if the highest `i` doesn't.
                    Some((constant, upper_bound))
                }
            }
            BinaryOp::Add { .. } | BinaryOp::Mul { .. } => {
                // `i + const` won't overflow if the highest `i` value doesn't.
                Some((constant, upper_bound))
            }
            BinaryOp::Div | BinaryOp::Mod => return SimplifyResult::None,
            _ => None,
        } {
            // We evaluate this expression using the upper bounds (or lower in the case of sub)
            // of its inputs to check whether it will ever overflow.
            // If `eval_constant_binary_op` won't overflow we can simplify the instruction to an unchecked version.
            let lhs = lhs.into_numeric_constant().0;
            let rhs = rhs.into_numeric_constant().0;
            match eval_constant_binary_op(lhs, rhs, binary.operator, operand_type) {
                BinaryEvaluationResult::Success(..) => {
                    // Unchecked version of the binary operation
                    let unchecked = Instruction::Binary(Binary {
                        operator: binary.operator.into_unchecked(),
                        lhs: binary.lhs,
                        rhs: binary.rhs,
                    });
                    return SimplifyResult::SimplifiedToInstruction(unchecked);
                }
                BinaryEvaluationResult::CouldNotEvaluate | BinaryEvaluationResult::Failure(..) => {
                    return SimplifyResult::None;
                }
            }
        }

        // Handle comparisons. (The upper_bound is exclusive).
        match binary.operator {
            BinaryOp::Eq => {
                if constant >= upper_bound || constant < lower_bound {
                    // `i == const` cannot be true if the constant is out of range.
                    SimplifyResult::SimplifiedTo(self.false_value)
                } else {
                    SimplifyResult::None
                }
            }
            BinaryOp::Lt => match is_induction_var_lhs {
                // `i < const`
                true if upper_bound <= constant => SimplifyResult::SimplifiedTo(self.true_value),
                true if lower_bound >= constant => SimplifyResult::SimplifiedTo(self.false_value),
                // `const < i`
                false if lower_bound > constant => SimplifyResult::SimplifiedTo(self.true_value),
                false if upper_bound <= constant.inc() => {
                    // If `const >= upper_bound - 1` then it will never be less than `i`.
                    SimplifyResult::SimplifiedTo(self.false_value)
                }
                _ => SimplifyResult::None,
            },
            _ => SimplifyResult::None,
        }
    }
}
