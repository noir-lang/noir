use acvm::{
    acir::{
        circuit::opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
        native_types::Expression,
    },
    FieldElement,
};

use crate::{
    ssa::{
        acir_gen::{constraints, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        node::BinaryOp,
    },
    Evaluator,
};

pub(super) fn simplify_bitwise(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    opcode: &BinaryOp,
) -> Option<InternalVar> {
    // Simplifies Bitwise operations of the form `a OP a`
    // where `a` is an integer
    //
    // a XOR a == 0
    // a AND a == a
    // a OR  a == a
    if lhs == rhs {
        return Some(match opcode {
            BinaryOp::And => lhs.clone(),
            BinaryOp::Or => lhs.clone(),
            BinaryOp::Xor => InternalVar::from(FieldElement::zero()),
            _ => unreachable!("This method should only be called on bitwise binary operators"),
        });
    }

    assert!(bit_size < FieldElement::max_num_bits());
    let max = FieldElement::from((1_u128 << bit_size) - 1);

    let (field, var) = match (lhs.to_const(), rhs.to_const()) {
        (Some(l_c), None) => (l_c.is_zero() || l_c == max).then_some((l_c, rhs))?,
        (None, Some(r_c)) => (r_c.is_zero() || r_c == max).then_some((r_c, lhs))?,
        _ => return None,
    };

    //simplify bitwise operation of the form: 0 OP var or 1 OP var
    Some(match opcode {
        BinaryOp::And => {
            if field.is_zero() {
                InternalVar::from(field)
            } else {
                var.clone()
            }
        }
        BinaryOp::Xor => {
            if field.is_zero() {
                var.clone()
            } else {
                InternalVar::from(constraints::subtract(
                    &Expression::from_field(field),
                    FieldElement::one(),
                    var.expression(),
                ))
            }
        }
        BinaryOp::Or => {
            if field.is_zero() {
                var.clone()
            } else {
                InternalVar::from(field)
            }
        }
        _ => unreachable!(),
    })
}
// Precondition: `lhs` and `rhs` do not represent constant expressions
pub(super) fn evaluate_bitwise(
    lhs: InternalVar,
    rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
    var_cache: &mut InternalVarCache,
    ctx: &SsaContext,
    opcode: BinaryOp,
) -> Expression {
    // Check precondition
    if let (Some(_), Some(_)) = (lhs.to_const(), rhs.to_const()) {
        unreachable!("ICE: `lhs` and `rhs` are expected to be simplified. Therefore it should not be possible for both to be constants.");
    }

    if bit_size == 1 {
        match opcode {
            BinaryOp::And => {
                return constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression())
            }
            BinaryOp::Xor => {
                let sum = constraints::add(lhs.expression(), FieldElement::one(), rhs.expression());
                let mul =
                    constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression());
                return constraints::subtract(&sum, FieldElement::from(2_i128), &mul);
            }
            BinaryOp::Or => {
                let sum = constraints::add(lhs.expression(), FieldElement::one(), rhs.expression());
                let mul =
                    constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression());
                return constraints::subtract(&sum, FieldElement::one(), &mul);
            }
            _ => unreachable!(),
        }
    }
    //We generate witness from const values in order to use the ACIR bitwise gates
    // If the gate is implemented, it is expected to be better than going through bit decomposition, even if one of the operand is a constant
    // If the gate is not implemented, we rely on the ACIR simplification to remove these witnesses
    //
    let mut a_witness = var_cache.get_or_compute_witness_unwrap(lhs, evaluator, ctx);
    let mut b_witness = var_cache.get_or_compute_witness_unwrap(rhs, evaluator, ctx);

    let result = evaluator.add_witness_to_cs();
    let bit_size = if bit_size % 2 == 1 { bit_size + 1 } else { bit_size };
    assert!(bit_size < FieldElement::max_num_bits() - 1);
    let max = FieldElement::from((1_u128 << bit_size) - 1);
    let gate = match opcode {
        BinaryOp::And => AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: a_witness, num_bits: bit_size },
            rhs: FunctionInput { witness: b_witness, num_bits: bit_size },
            output: result,
        }),
        BinaryOp::Xor => AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
            lhs: FunctionInput { witness: a_witness, num_bits: bit_size },
            rhs: FunctionInput { witness: b_witness, num_bits: bit_size },
            output: result,
        }),
        BinaryOp::Or => {
            a_witness = evaluator.create_intermediate_variable(constraints::subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                &Expression::from(a_witness),
            ));
            b_witness = evaluator.create_intermediate_variable(constraints::subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                &Expression::from(b_witness),
            ));
            // We do not have an OR gate yet, so we use the AND gate
            AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
                lhs: FunctionInput { witness: a_witness, num_bits: bit_size },
                rhs: FunctionInput { witness: b_witness, num_bits: bit_size },
                output: result,
            })
        }
        _ => unreachable!("ICE: expected a bitwise operation"),
    };

    evaluator.opcodes.push(gate);

    if opcode == BinaryOp::Or {
        constraints::subtract(
            &Expression::from_field(max),
            FieldElement::one(),
            &Expression::from(result),
        )
    } else {
        Expression::from(result)
    }
}
