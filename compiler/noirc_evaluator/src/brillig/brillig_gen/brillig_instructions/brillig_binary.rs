use acvm::{AcirField, FieldElement};

use crate::brillig::brillig_gen::brillig_block::{BrilligBlock, type_of_binary_operation};
use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
use crate::brillig::brillig_ir::brillig_variable::SingleAddrVariable;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::brillig::brillig_ir::{BrilligBinaryOp, BrilligContext};
use crate::ssa::ir::instruction::{BinaryOp, InstructionId, binary::Binary};
use crate::ssa::ir::types::{NumericType, Type};
use crate::ssa::ir::{dfg::DataFlowGraph, instruction::ConstrainError, value::ValueId};
use iter_extended::vecmap;

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Converts the Binary instruction into a sequence of Brillig opcodes.
    fn convert_ssa_binary(
        &mut self,
        binary: &Binary,
        dfg: &DataFlowGraph,
        result_variable: SingleAddrVariable,
    ) {
        let binary_type = type_of_binary_operation(
            dfg[binary.lhs].get_type().as_ref(),
            dfg[binary.rhs].get_type().as_ref(),
            binary.operator,
        );

        let left = self.convert_ssa_single_addr_value(binary.lhs, dfg);
        let right = self.convert_ssa_single_addr_value(binary.rhs, dfg);

        let (is_field, is_signed) = match binary_type {
            Type::Numeric(numeric_type) => match numeric_type {
                NumericType::Signed { .. } => (false, true),
                NumericType::Unsigned { .. } => (false, false),
                NumericType::NativeField => (true, false),
            },
            _ => unreachable!(
                "only numeric types are allowed in binary operations. References are handled separately"
            ),
        };

        let brillig_binary_op = match binary.operator {
            BinaryOp::Div => {
                if is_signed {
                    self.brillig_context.convert_signed_division(left, right, result_variable);
                    return;
                } else if is_field {
                    BrilligBinaryOp::FieldDiv
                } else {
                    BrilligBinaryOp::UnsignedDiv
                }
            }
            BinaryOp::Mod => {
                if is_signed {
                    self.convert_signed_modulo(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::Modulo
                }
            }
            BinaryOp::Add { .. } => BrilligBinaryOp::Add,
            BinaryOp::Sub { .. } => BrilligBinaryOp::Sub,
            BinaryOp::Mul { .. } => BrilligBinaryOp::Mul,
            BinaryOp::Eq => BrilligBinaryOp::Equals,
            BinaryOp::Lt => {
                if is_signed {
                    self.convert_signed_less_than(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::LessThan
                }
            }
            BinaryOp::And => BrilligBinaryOp::And,
            BinaryOp::Or => BrilligBinaryOp::Or,
            BinaryOp::Xor => BrilligBinaryOp::Xor,
            BinaryOp::Shl => BrilligBinaryOp::Shl,
            BinaryOp::Shr => {
                if is_signed {
                    self.convert_signed_shr(left, right, result_variable);
                    return;
                } else {
                    BrilligBinaryOp::Shr
                }
            }
        };

        self.brillig_context.binary_instruction(left, right, result_variable, brillig_binary_op);

        self.add_overflow_check(left, right, result_variable, binary, dfg, is_signed);
    }

    fn convert_signed_modulo(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let scratch_var_i =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);
        let scratch_var_j =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);

        // i = left / right
        self.brillig_context.convert_signed_division(left, right, scratch_var_i);

        // j = i * right
        self.brillig_context.binary_instruction(
            scratch_var_i,
            right,
            scratch_var_j,
            BrilligBinaryOp::Mul,
        );

        // result_register = left - j
        self.brillig_context.binary_instruction(left, scratch_var_j, result, BrilligBinaryOp::Sub);
        // Free scratch registers
        self.brillig_context.deallocate_single_addr(scratch_var_i);
        self.brillig_context.deallocate_single_addr(scratch_var_j);
    }

    fn convert_signed_shr(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        // Check if left is negative
        let left_is_negative = SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
        let max_positive = self
            .brillig_context
            .make_constant_instruction(((1_u128 << (left.bit_size - 1)) - 1).into(), left.bit_size);
        self.brillig_context.binary_instruction(
            max_positive,
            left,
            left_is_negative,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context.codegen_branch(left_is_negative.address, |ctx, is_negative| {
            if is_negative {
                let one = ctx.make_constant_instruction(1_u128.into(), left.bit_size);

                // computes 2^right
                let two_pow = SingleAddrVariable::new(ctx.allocate_register(), left.bit_size);
                ctx.binary_instruction(one, right, two_pow, BrilligBinaryOp::Shl);

                // Right shift using division on 1-complement
                ctx.binary_instruction(left, one, result, BrilligBinaryOp::Add);
                ctx.convert_signed_division(result, two_pow, result);
                ctx.binary_instruction(result, one, result, BrilligBinaryOp::Sub);

                // Clean-up
                ctx.deallocate_single_addr(one);
                ctx.deallocate_single_addr(two_pow);
            } else {
                ctx.binary_instruction(left, right, result, BrilligBinaryOp::Shr);
            }
        });

        self.brillig_context.deallocate_single_addr(left_is_negative);
    }

    fn convert_signed_less_than(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let biased_left =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), left.bit_size);
        let biased_right =
            SingleAddrVariable::new(self.brillig_context.allocate_register(), right.bit_size);

        let bias = self
            .brillig_context
            .make_constant_instruction((1_u128 << (left.bit_size - 1)).into(), left.bit_size);

        self.brillig_context.binary_instruction(left, bias, biased_left, BrilligBinaryOp::Add);
        self.brillig_context.binary_instruction(right, bias, biased_right, BrilligBinaryOp::Add);

        self.brillig_context.binary_instruction(
            biased_left,
            biased_right,
            result,
            BrilligBinaryOp::LessThan,
        );

        self.brillig_context.deallocate_single_addr(biased_left);
        self.brillig_context.deallocate_single_addr(biased_right);
        self.brillig_context.deallocate_single_addr(bias);
    }

    /// Overflow checks for the following unsigned binary operations
    /// - Checked Add/Sub/Mul
    #[allow(clippy::too_many_arguments)]
    fn add_overflow_check(
        &mut self,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        result: SingleAddrVariable,
        binary: &Binary,
        dfg: &DataFlowGraph,
        is_signed: bool,
    ) {
        let bit_size = left.bit_size;

        if bit_size == FieldElement::max_num_bits() || is_signed {
            if is_signed
                && matches!(
                    binary.operator,
                    BinaryOp::Add { unchecked: false }
                        | BinaryOp::Sub { unchecked: false }
                        | BinaryOp::Mul { unchecked: false }
                )
            {
                panic!("Checked signed operations should all be removed before brillig-gen")
            }
            return;
        }

        match binary.operator {
            BinaryOp::Add { unchecked: false } => {
                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that lhs <= result
                self.brillig_context.binary_instruction(
                    left,
                    result,
                    condition,
                    BrilligBinaryOp::LessThanEquals,
                );
                let msg = "attempt to add with overflow".to_string();
                self.brillig_context.codegen_constrain(condition, Some(msg));
                self.brillig_context.deallocate_single_addr(condition);
            }
            BinaryOp::Sub { unchecked: false } => {
                let condition =
                    SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                // Check that rhs <= lhs
                self.brillig_context.binary_instruction(
                    right,
                    left,
                    condition,
                    BrilligBinaryOp::LessThanEquals,
                );
                let msg = "attempt to subtract with overflow".to_string();
                self.brillig_context.codegen_constrain(condition, Some(msg));
                self.brillig_context.deallocate_single_addr(condition);
            }
            BinaryOp::Mul { unchecked: false } => {
                let division_by_rhs_gives_lhs = |ctx: &mut BrilligContext<
                    FieldElement,
                    Registers,
                >| {
                    let condition = SingleAddrVariable::new(ctx.allocate_register(), 1);
                    let division = SingleAddrVariable::new(ctx.allocate_register(), bit_size);
                    // Check that result / rhs == lhs
                    ctx.binary_instruction(result, right, division, BrilligBinaryOp::UnsignedDiv);
                    ctx.binary_instruction(division, left, condition, BrilligBinaryOp::Equals);
                    let msg = "attempt to multiply with overflow".to_string();
                    ctx.codegen_constrain(condition, Some(msg));
                    ctx.deallocate_single_addr(condition);
                    ctx.deallocate_single_addr(division);
                };

                let rhs_may_be_zero =
                    dfg.get_numeric_constant(binary.rhs).is_none_or(|rhs| rhs.is_zero());
                if rhs_may_be_zero {
                    let is_right_zero =
                        SingleAddrVariable::new(self.brillig_context.allocate_register(), 1);
                    let zero =
                        self.brillig_context.make_constant_instruction(0_usize.into(), bit_size);
                    self.brillig_context.binary_instruction(
                        zero,
                        right,
                        is_right_zero,
                        BrilligBinaryOp::Equals,
                    );
                    self.brillig_context
                        .codegen_if_not(is_right_zero.address, division_by_rhs_gives_lhs);
                    self.brillig_context.deallocate_single_addr(is_right_zero);
                    self.brillig_context.deallocate_single_addr(zero);
                } else {
                    division_by_rhs_gives_lhs(self.brillig_context);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn binary_gen(
        &mut self,
        instruction_id: InstructionId,
        binary: &Binary,
        dfg: &DataFlowGraph,
    ) {
        let [result_value] = dfg.instruction_result(instruction_id);
        let result_var = self.variables.define_single_addr_variable(
            self.function_context,
            self.brillig_context,
            result_value,
            dfg,
        );
        self.convert_ssa_binary(binary, dfg, result_var);
    }

    pub(crate) fn constrain_gen(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        assert_message: &Option<ConstrainError>,
        dfg: &DataFlowGraph,
    ) {
        let (condition, deallocate) = match (
            dfg.get_numeric_constant_with_type(lhs),
            dfg.get_numeric_constant_with_type(rhs),
        ) {
            // If the constraint is of the form `x == u1 1` then we can simply constrain `x` directly
            (Some((constant, NumericType::Unsigned { bit_size: 1 })), None)
                if constant == FieldElement::one() =>
            {
                (self.convert_ssa_single_addr_value(rhs, dfg), false)
            }
            (None, Some((constant, NumericType::Unsigned { bit_size: 1 })))
                if constant == FieldElement::one() =>
            {
                (self.convert_ssa_single_addr_value(lhs, dfg), false)
            }

            // Otherwise we need to perform the equality explicitly.
            _ => {
                let condition = SingleAddrVariable {
                    address: self.brillig_context.allocate_register(),
                    bit_size: 1,
                };
                self.convert_ssa_binary(
                    &Binary { lhs, rhs, operator: BinaryOp::Eq },
                    dfg,
                    condition,
                );

                (condition, true)
            }
        };

        match assert_message {
            Some(ConstrainError::Dynamic(selector, _, values)) => {
                let payload_values = vecmap(values, |value| self.convert_ssa_value(*value, dfg));
                let payload_as_params = vecmap(values, |value| {
                    let value_type = dfg.type_of_value(*value);
                    FunctionContext::ssa_type_to_parameter(&value_type)
                });
                self.brillig_context.codegen_constrain_with_revert_data(
                    condition,
                    payload_values,
                    payload_as_params,
                    *selector,
                );
            }
            Some(ConstrainError::StaticString(message)) => {
                self.brillig_context.codegen_constrain(condition, Some(message.clone()));
            }
            None => {
                self.brillig_context.codegen_constrain(condition, None);
            }
        }
        if deallocate {
            self.brillig_context.deallocate_single_addr(condition);
        }
    }
}
