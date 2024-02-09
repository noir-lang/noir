use acvm::FieldElement;

use super::{Binary, BinaryOp, ConstrainError, DataFlowGraph, Instruction, Type, Value, ValueId};

/// Try to decompose this constrain instruction. This constraint will be broken down such that it instead constrains
/// all the values which are used to compute the values which were being constrained.
pub(super) fn decompose_constrain(
    lhs: ValueId,
    rhs: ValueId,
    msg: &Option<Box<ConstrainError>>,
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

            (Value::NumericConstant { constant, typ }, Value::Instruction { instruction, .. })
            | (Value::Instruction { instruction, .. }, Value::NumericConstant { constant, typ })
                if typ.is_native_field() | typ.is_unsigned() =>
            {
                match dfg[*instruction] {
                    Instruction::Binary(Binary { lhs: binary_lhs, rhs: binary_rhs, operator })
                        if dfg.is_constant(binary_lhs) || dfg.is_constant(binary_rhs) =>
                    {
                        // Replace an assertion on the output of a binary instruction
                        //
                        // v2 = add v0, Field 1
                        // constrain v2 == Field 3
                        //
                        // with an assertion in terms of the non-constant input to the binary instruction
                        //
                        // v2 = add v0, Field 1
                        // constrain v0 == Field 2
                        //
                        // Note that this doesn't remove the value `v2` as it may be used in other instructions, but it
                        // will likely be removed through dead instruction elimination.

                        fn calculate_binary_input(
                            operator: BinaryOp,
                            result: FieldElement,
                            known_input: FieldElement,
                            typ: &Type,
                            lhs_is_known: bool,
                        ) -> Option<FieldElement> {
                            match operator {
                                BinaryOp::Add => Some(result - known_input),
                                BinaryOp::Sub => {
                                    if lhs_is_known {
                                        Some(known_input - result)
                                    } else {
                                        Some(result + known_input)
                                    }
                                }
                                BinaryOp::Mul => {
                                    if typ.is_native_field() {
                                        Some(result / known_input)
                                    } else {
                                        // TODO: simplify integer division
                                        if result == known_input {
                                            Some(FieldElement::one())
                                        } else {
                                            None
                                        }
                                    }
                                }
                                BinaryOp::Div => {
                                    if typ.is_native_field() {
                                        if lhs_is_known {
                                            // k / x == r => x == k / r
                                            Some(known_input / result)
                                        } else {
                                            // x / k == r => x == k * r
                                            Some(known_input * result)
                                        }
                                    } else {
                                        None
                                    }
                                }

                                BinaryOp::Xor => Some(result.xor(&known_input, typ.bit_size())),

                                BinaryOp::Eq => {
                                    unreachable!("This should be handled by the boolean solver")
                                }
                                BinaryOp::Mod | BinaryOp::Lt | BinaryOp::And | BinaryOp::Or => None, // These operations lose information so can't be reversed.
                            }
                        }

                        let (variable, value) = match (
                            dfg.get_numeric_constant(binary_lhs),
                            dfg.get_numeric_constant(binary_rhs),
                        ) {
                            (Some(known_input), _) => (
                                binary_rhs,
                                calculate_binary_input(operator, *constant, known_input, typ, true),
                            ),
                            (_, Some(known_input)) => (
                                binary_lhs,
                                calculate_binary_input(
                                    operator,
                                    *constant,
                                    known_input,
                                    typ,
                                    false,
                                ),
                            ),
                            _ => {
                                unreachable!("Checked that one of the addition inputs is constant")
                            }
                        };

                        if let Some(value) = value {
                            let value = dfg.make_constant(value, typ.clone());
                            vec![Instruction::Constrain(variable, value, msg.clone())]
                        } else {
                            vec![Instruction::Constrain(lhs, rhs, msg.clone())]
                        }
                    }

                    _ => vec![Instruction::Constrain(lhs, rhs, msg.clone())],
                }
            }

            _ => vec![Instruction::Constrain(lhs, rhs, msg.clone())],
        }
    }
}
