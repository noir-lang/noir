use acvm::{acir::AcirField, FieldElement};

use super::{Binary, BinaryOp, ConstrainError, DataFlowGraph, Instruction, Type, Value, ValueId};

/// Try to decompose this constrain instruction. This constraint will be broken down such that it instead constrains
/// all the values which are used to compute the values which were being constrained.
pub(super) fn decompose_constrain(
    lhs: ValueId,
    rhs: ValueId,
    msg: &Option<ConstrainError>,
    dfg: &mut DataFlowGraph,
) -> Vec<Instruction> {
    let lhs = dfg.resolve(lhs);
    let rhs = dfg.resolve(rhs);

    if lhs == rhs {
        // Remove trivial case `assert_eq(x, x)`
        Vec::new()
    } else {
        match (&dfg[lhs], &dfg[rhs]) {
            (Value::NumericConstant { constant, typ }, Value::Instruction { instruction, .. })
            | (Value::Instruction { instruction, .. }, Value::NumericConstant { constant, typ })
                if *typ == Type::bool() =>
            {
                match dfg[*instruction] {
                    Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Eq })
                        if constant.is_one() =>
                    {
                        // Replace an explicit two step equality assertion
                        //
                        // v2 = eq v0, u32 v1
                        // constrain v2 == u1 1
                        //
                        // with a direct assertion of equality between the two values
                        //
                        // v2 = eq v0, u32 v1
                        // constrain v0 == v1
                        //
                        // Note that this doesn't remove the value `v2` as it may be used in other instructions, but it
                        // will likely be removed through dead instruction elimination.

                        vec![Instruction::Constrain(lhs, rhs, msg.clone())]
                    }

                    Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Mul })
                        if constant.is_one() && dfg.type_of_value(lhs) == Type::bool() =>
                    {
                        // Replace an equality assertion on a boolean multiplication
                        //
                        // v2 = mul v0, v1
                        // constrain v2 == u1 1
                        //
                        // with a direct assertion that each value is equal to 1
                        //
                        // v2 = mul v0, v1
                        // constrain v0 == 1
                        // constrain v1 == 1
                        //
                        // This is due to the fact that for `v2` to be 1 then both `v0` and `v1` are 1.
                        //
                        // Note that this doesn't remove the value `v2` as it may be used in other instructions, but it
                        // will likely be removed through dead instruction elimination.
                        let one = FieldElement::one();
                        let one = dfg.make_constant(one, Type::bool());

                        [
                            decompose_constrain(lhs, one, msg, dfg),
                            decompose_constrain(rhs, one, msg, dfg),
                        ]
                        .concat()
                    }

                    Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Or })
                        if constant.is_zero() =>
                    {
                        // Replace an equality assertion on an OR
                        //
                        // v2 = or v0, v1
                        // constrain v2 == u1 0
                        //
                        // with a direct assertion that each value is equal to 0
                        //
                        // v2 = or v0, v1
                        // constrain v0 == 0
                        // constrain v1 == 0
                        //
                        // This is due to the fact that for `v2` to be 0 then both `v0` and `v1` are 0.
                        //
                        // Note that this doesn't remove the value `v2` as it may be used in other instructions, but it
                        // will likely be removed through dead instruction elimination.
                        let zero = FieldElement::zero();
                        let zero = dfg.make_constant(zero, dfg.type_of_value(lhs));

                        [
                            decompose_constrain(lhs, zero, msg, dfg),
                            decompose_constrain(rhs, zero, msg, dfg),
                        ]
                        .concat()
                    }

                    Instruction::Not(value) => {
                        // Replace an assertion that a not instruction is truthy
                        //
                        // v1 = not v0
                        // constrain v1 == u1 1
                        //
                        // with an assertion that the not instruction input is falsy
                        //
                        // v1 = not v0
                        // constrain v0 == u1 0
                        //
                        // Note that this doesn't remove the value `v1` as it may be used in other instructions, but it
                        // will likely be removed through dead instruction elimination.
                        let reversed_constant = FieldElement::from(!constant.is_one());
                        let reversed_constant = dfg.make_constant(reversed_constant, Type::bool());
                        decompose_constrain(value, reversed_constant, msg, dfg)
                    }

                    _ => vec![Instruction::Constrain(lhs, rhs, msg.clone())],
                }
            }

            (
                Value::Instruction { instruction: instruction_lhs, .. },
                Value::Instruction { instruction: instruction_rhs, .. },
            ) => {
                match (&dfg[*instruction_lhs], &dfg[*instruction_rhs]) {
                    // Casting two values just to enforce an equality on them.
                    //
                    // This is equivalent to enforcing equality on the original values.
                    (Instruction::Cast(original_lhs, _), Instruction::Cast(original_rhs, _))
                        if dfg.type_of_value(*original_lhs) == dfg.type_of_value(*original_rhs) =>
                    {
                        vec![Instruction::Constrain(*original_lhs, *original_rhs, msg.clone())]
                    }

                    _ => vec![Instruction::Constrain(lhs, rhs, msg.clone())],
                }
            }
            _ => vec![Instruction::Constrain(lhs, rhs, msg.clone())],
        }
    }
}
