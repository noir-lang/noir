use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        types::NumericType,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// An SSA pass that will turn checked binary addition, subtraction and multiplication into
    /// unchecked ones if it's guaranteed that the operations cannot overflow.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn checked_to_unchecked(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.checked_to_unchecked();
        }
        self
    }
}

impl Function {
    fn checked_to_unchecked(&mut self) {
        self.simple_reachable_blocks_optimization(|context| {
            let instruction = context.instruction();
            let Instruction::Binary(binary) = instruction else {
                return;
            };
            let lhs = binary.lhs;
            let rhs = binary.rhs;

            let lhs_type = context.dfg.type_of_value(lhs).unwrap_numeric();
            let NumericType::Unsigned { .. } = lhs_type else {
                return;
            };

            let dfg = &context.dfg;

            match binary.operator {
                BinaryOp::Add { unchecked: false } => {
                    let bit_size = dfg.type_of_value(lhs).bit_size();

                    if dfg.get_value_max_num_bits(lhs) < bit_size
                        && dfg.get_value_max_num_bits(rhs) < bit_size
                    {
                        // `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                        let operator = BinaryOp::Add { unchecked: true };
                        let binary = Binary { operator, ..*binary };
                        context.replace_current_instruction_with(Instruction::Binary(binary));
                    }
                }
                BinaryOp::Sub { unchecked: false } => {
                    if dfg.is_constant(lhs)
                        && dfg.get_value_max_num_bits(lhs) > dfg.get_value_max_num_bits(rhs)
                    {
                        // `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs > 0`
                        // Note strict inequality as `rhs > lhs` while `max_lhs_bits == max_rhs_bits` is possible.
                        let operator = BinaryOp::Sub { unchecked: true };
                        let binary = Binary { operator, ..*binary };
                        context.replace_current_instruction_with(Instruction::Binary(binary));
                    }
                }
                BinaryOp::Mul { unchecked: false } => {
                    let bit_size = dfg.type_of_value(lhs).bit_size();
                    let max_lhs_bits = dfg.get_value_max_num_bits(lhs);
                    let max_rhs_bits = dfg.get_value_max_num_bits(rhs);

                    if bit_size == 1
                        || max_lhs_bits + max_rhs_bits <= bit_size
                        || max_lhs_bits == 1
                        || max_rhs_bits == 1
                    {
                        // Either performing boolean multiplication (which cannot overflow),
                        // or `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                        let operator = BinaryOp::Mul { unchecked: true };
                        let binary = Binary { operator, ..*binary };
                        context.replace_current_instruction_with(Instruction::Binary(binary));
                    }
                }
                _ => (),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    #[test]
    fn checked_to_unchecked_when_casting_two_u16_to_u32_then_adding() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u16, v1: u16):
            v2 = cast v0 as u32
            v3 = cast v1 as u32
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u16, v1: u16):
            v2 = cast v0 as u32
            v3 = cast v1 as u32
            v4 = unchecked_add v2, v3
            return v4
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_subtracting_u32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u16):
            v1 = cast v0 as u32
            v2 = sub u32 65536, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u16):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 65536, v1
            return v3
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_multiplying_bools() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = mul v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = unchecked_mul v0, v1
            return v2
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_multiplying_upcasted_bool_with_u32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            v2 = cast v0 as u32
            v3 = mul v2, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            v2 = cast v0 as u32
            v3 = unchecked_mul v2, v1
            return v2
        }
        ");
    }

    #[test]
    fn no_checked_to_unchecked_when_casting_two_i16_to_i32_then_adding() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i16, v1: i16):
            v2 = cast v0 as i32
            v3 = cast v1 as i32
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn no_checked_to_unchecked_when_subtracting_i32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i16):
            v1 = cast v0 as i32
            v2 = sub i32 65536, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn no_checked_to_unchecked_when_multiplying_upcasted_bool_with_i32() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: i32):
            v2 = cast v0 as i32
            v3 = mul v2, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_normalized_ssa_equals(ssa, src);
    }
}
