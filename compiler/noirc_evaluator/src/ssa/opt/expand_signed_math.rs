use acvm::FieldElement;

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Expands signed "less than" operations in ACIR to be done using unsigned operations.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn expand_signed_math(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.expand_signed_math();
        }
        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions, decomposing any checked signed "less than" operations
    /// to be done using unsigned types, but only if this is an ACIR function.
    fn expand_signed_math(&mut self) {
        if !self.dfg.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            // We only care about "less than"
            let Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Lt }) = instruction
            else {
                return;
            };

            // ... and it must be a signed integer operation.
            if !context.dfg.type_of_value(*lhs).is_signed() {
                return;
            }

            let lhs = *lhs;
            let rhs = *rhs;

            // We remove the current instruction, as we will need to replace it with multiple new instructions.
            context.remove_current_instruction();

            let [old_result] = context.dfg.instruction_result(instruction_id);

            let mut expansion_context = Context { context };
            let new_result = expansion_context.insert_lt(lhs, rhs);

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        expand_signed_math_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
}

impl Context<'_, '_, '_> {
    fn insert_lt(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        // First cast lhs and rhs to their unsigned equivalents
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unsigned_typ = NumericType::unsigned(bit_size);
        let lhs_unsigned = self.insert_cast(lhs, unsigned_typ);
        let rhs_unsigned = self.insert_cast(rhs, unsigned_typ);

        // Check if lhs and rhs are positive or negative, respectively
        let pow_last = self.numeric_constant(1_u128 << (bit_size - 1), unsigned_typ);
        let lhs_is_positive = self.insert_binary(lhs_unsigned, BinaryOp::Div, pow_last);
        let lhs_is_positive = self.insert_cast(lhs_is_positive, NumericType::bool());
        let rhs_is_positive = self.insert_binary(rhs_unsigned, BinaryOp::Div, pow_last);
        let rhs_is_positive = self.insert_cast(rhs_is_positive, NumericType::bool());

        // Do rhs and lhs have a different sign?
        let different_sign = self.insert_binary(lhs_is_positive, BinaryOp::Xor, rhs_is_positive);

        // Check lhs < rhs using their unsigned equivalents
        let unsigned_lt = self.insert_binary(lhs_unsigned, BinaryOp::Lt, rhs_unsigned);

        // It can be shown that the result is given by xor'ing the two results above:
        // - if lhs and rhs have the same sign (different_sign is 0):
        //   - if both are positive then the unsigned comparison is correct, xoring it with 0 gives
        //     same result
        //   - if both are negative then the unsigned comparison is also correct, as, for example,
        //     for i8, -128 i8 is Field 128 and -1 i8 is Field 255 and `-128 < -1` and `128 < 255`
        // - if lhs and rhs have different signs (different_sign is 1):
        //   - if lhs is positive and rhs is negative then, as fields, rhs will be greater, but
        //     the result is the opposite (so xor'ing with 1 gives the correct result)
        //   - if lhs is negative and rhs is positive then, as fields, lhs will be greater, but
        //     the result is the opposite (so xor'ing with 1 gives the correct result)
        self.insert_binary(different_sign, BinaryOp::Xor, unsigned_lt)
    }

    /// Insert a numeric constant into the current function
    fn numeric_constant(&mut self, value: impl Into<FieldElement>, typ: NumericType) -> ValueId {
        self.context.dfg.make_constant(value.into(), typ)
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    fn insert_binary(&mut self, lhs: ValueId, operator: BinaryOp, rhs: ValueId) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.context.insert_instruction(instruction, None).first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.context.insert_instruction(Instruction::Cast(value, typ), None).first()
    }
}

/// Post-check condition for [Function::expand_signed_math].
///
/// Succeeds if:
///   - `func` does not contain any signed "less than" ops
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn expand_signed_math_post_check(func: &Function) {
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if let Instruction::Binary(binary) = &func.dfg[*instruction_id] {
                if func.dfg.type_of_value(binary.lhs).is_signed() {
                    match binary.operator {
                        BinaryOp::Lt => {
                            panic!("Checked signed 'less than' has not been removed")
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn expands_signed_lt_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = lt v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_math();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = cast v0 as u8
            v3 = cast v1 as u8
            v5 = div v2, u8 128
            v6 = cast v5 as u1
            v7 = div v3, u8 128
            v8 = cast v7 as u1
            v9 = xor v6, v8
            v10 = lt v2, v3
            v11 = xor v9, v10
            return v11
        }
        ");
    }

    #[test]
    fn does_not_expand_signed_lt_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = lt v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_math);
    }
}
