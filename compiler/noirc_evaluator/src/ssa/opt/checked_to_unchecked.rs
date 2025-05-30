use acvm::AcirField as _;
use fxhash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        types::NumericType,
        value::{Value, ValueId},
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
        let mut value_max_num_bits = HashMap::<ValueId, u32>::default();

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
                    let max_lhs_bits = get_max_num_bits(dfg, lhs, &mut value_max_num_bits);
                    let max_rhs_bits = get_max_num_bits(dfg, rhs, &mut value_max_num_bits);

                    if max_lhs_bits < bit_size && max_rhs_bits < bit_size {
                        // `lhs` and `rhs` have both been casted up from smaller types and so cannot overflow.
                        let operator = BinaryOp::Add { unchecked: true };
                        let binary = Binary { operator, ..*binary };
                        context.replace_current_instruction_with(Instruction::Binary(binary));
                    }
                }
                BinaryOp::Sub { unchecked: false } => {
                    let Some(lhs_const) = dfg.get_numeric_constant(lhs) else {
                        return;
                    };

                    let max_lhs_bits = get_max_num_bits(dfg, lhs, &mut value_max_num_bits);
                    let max_rhs_bits = get_max_num_bits(dfg, rhs, &mut value_max_num_bits);
                    let max_rhs =
                        if max_rhs_bits == 128 { u128::MAX } else { (1 << max_rhs_bits) - 1 };

                    // 1. `lhs` is a fixed constant and `rhs` is restricted such that `lhs - rhs > 0`
                    // Note strict inequality as `rhs > lhs` while `max_lhs_bits == max_rhs_bits` is possible.
                    // 2. `lhs` is the maximum value for the maximum bitsize of `rhs`.
                    //    For example: `lhs` is 1 and `rhs` max bitsize is 1, so at most it's `1 - 1` which cannot overflow.
                    //    Another example: `lhs` is 255 and `rhs` max bitsize is 8, so at most it's `255 - 255` which cannot overflow, etc.
                    if max_lhs_bits > max_rhs_bits || (lhs_const == max_rhs.into()) {
                        let operator = BinaryOp::Sub { unchecked: true };
                        let binary = Binary { operator, ..*binary };
                        context.replace_current_instruction_with(Instruction::Binary(binary));
                    }
                }
                BinaryOp::Mul { unchecked: false } => {
                    let bit_size = dfg.type_of_value(lhs).bit_size();
                    let max_lhs_bits = get_max_num_bits(dfg, lhs, &mut value_max_num_bits);
                    let max_rhs_bits = get_max_num_bits(dfg, rhs, &mut value_max_num_bits);

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

/// The logic here is almost the same as [`DataFlowGraph::get_value_max_num_bits`] except that
/// - it takes into account that the bitsize of multiplying two bools is 1
/// - it recurses by memoizing the results in `value_max_num_bits`
fn get_max_num_bits(
    dfg: &DataFlowGraph,
    value: ValueId,
    value_max_num_bits: &mut HashMap<ValueId, u32>,
) -> u32 {
    if let Some(bits) = value_max_num_bits.get(&value) {
        return *bits;
    }

    let value_bit_size = dfg.type_of_value(value).bit_size();

    let bits = match dfg[value] {
        Value::Instruction { instruction, .. } => {
            match dfg[instruction] {
                Instruction::Cast(original_value, _) => {
                    let original_bit_size =
                        get_max_num_bits(dfg, original_value, value_max_num_bits);
                    // We might have cast e.g. `u1` to `u8` to be able to do arithmetic,
                    // in which case we want to recover the original smaller bit size;
                    // OTOH if we cast down, then we don't need the higher original size.
                    value_bit_size.min(original_bit_size)
                }
                Instruction::Binary(Binary { lhs, operator: BinaryOp::Mul { .. }, rhs })
                    if get_max_num_bits(dfg, lhs, value_max_num_bits) == 1
                        && get_max_num_bits(dfg, rhs, value_max_num_bits) == 1 =>
                {
                    // When multiplying two values, if their bitsize is 1 then the result's bitsize will be 1 too
                    1
                }
                _ => value_bit_size,
            }
        }
        Value::NumericConstant { constant, .. } => constant.num_bits(),
        _ => value_bit_size,
    };

    assert!(bits <= value_bit_size);
    value_max_num_bits.insert(value, bits);

    bits
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
    fn checked_to_unchecked_when_subtracting_from_1_a_value_that_has_1_bit() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = cast v0 as u32
            v3 = sub u32 1, v1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 1, v1
            return v3
        }
        ");
    }

    #[test]
    fn checked_to_unchecked_when_subtracting_from_255_a_value_that_has_8_bits() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = cast v0 as u32
            v3 = sub u32 255, v1
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = cast v0 as u32
            v3 = unchecked_sub u32 255, v1
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
            v5 = truncate v4 to 32 bits, max_bit_size: 33
            return v5
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
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            return v3
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

    #[test]
    fn checked_to_unchecked_when_multiplying_two_upcasted_bools_to_u32_then_multiplying_again() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u32):
            v3 = cast v0 as u32
            v4 = cast v1 as u32
            v5 = mul v3, v4
            v6 = mul v2, v5
            return v6
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.checked_to_unchecked();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: u32):
            v3 = cast v0 as u32
            v4 = cast v1 as u32
            v5 = unchecked_mul v3, v4
            v6 = unchecked_mul v2, v5
            return v6
        }
        ");
    }
}
