use crate::{
    ssa::{
        acir_gen::{
            constraints, internal_var_cache::InternalVarCache, operations, Acir, InternalVar,
        },
        context::SsaContext,
        node::{self, BinaryOp, Node, ObjectType},
    },
    Evaluator,
};
use acvm::{acir::native_types::Expression, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

fn get_predicate(
    var_cache: &mut InternalVarCache,
    binary: &node::Binary,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> InternalVar {
    let predicate_node_id = match binary.predicate {
        Some(pred) => pred,
        None => return InternalVar::from(Expression::one()),
    };
    var_cache.get_or_compute_internal_var_unwrap(predicate_node_id, evaluator, ctx)
}

pub(crate) fn evaluate(
    binary: &node::Binary,
    res_type: ObjectType,
    acir_gen: &mut Acir,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    let r_size = ctx[binary.rhs].size_in_bits();
    let l_size = ctx[binary.lhs].size_in_bits();
    let max_size = u32::max(r_size, l_size);
    if binary.predicate == Some(ctx.zero()) {
        return None;
    }

    let binary_output = match &binary.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                InternalVar::from(constraints::add(
                    l_c.expression(),
                    FieldElement::one(),
                    r_c.expression(),
                ))
            },
            BinaryOp::Sub { max_rhs_value } | BinaryOp::SafeSub { max_rhs_value } => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                if res_type == ObjectType::native_field() {
                    InternalVar::from(constraints::subtract(
                        l_c.expression(),
                        FieldElement::one(),
                        r_c.expression(),
                    ))
                } else {
                    //we need the type of rhs and its max value, then:
                    //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                    let bit_size = ctx[binary.rhs].get_type().bits();
                    let r_big = BigUint::one() << bit_size;
                    let mut k = max_rhs_value / &r_big;
                    if max_rhs_value % &r_big != BigUint::zero() {
                        k = &k + BigUint::one();
                    }
                    k = &k * r_big;
                    let f = FieldElement::from_be_bytes_reduce(&k.to_bytes_be());
                    let mut sub_expr = constraints::subtract(
                        l_c.expression(),
                        FieldElement::one(),
                        r_c.expression(),
                    );
                    sub_expr.q_c += f;
                    let mut sub_var = sub_expr.into();
                    //TODO: uses interval analysis for more precise check
                    if let Some(lhs_const) = l_c.to_const() {
                        if max_rhs_value <= &BigUint::from_bytes_be(&lhs_const.to_be_bytes()) {
                            sub_var = InternalVar::from(constraints::subtract(
                                l_c.expression(),
                                FieldElement::one(),
                                r_c.expression(),
                            ));
                        }
                    }
                    sub_var
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                InternalVar::from(constraints::mul_with_witness(
                evaluator,
                l_c.expression(),
                r_c.expression(),
            ))
            },
            BinaryOp::Udiv(_) => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let predicate = get_predicate(&mut acir_gen.var_cache,binary, evaluator, ctx);
                let (q_wit, _) = constraints::evaluate_udiv(
                    l_c.expression(),
                    r_c.expression(),
                    max_size,
                    predicate.expression(),
                    evaluator,
                );
                InternalVar::from(q_wit)
            }
            BinaryOp::Sdiv(_) => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                InternalVar::from(
                constraints::evaluate_sdiv(l_c.expression(), r_c.expression(), evaluator).0,
            )
        },
            BinaryOp::Urem(_) => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let predicate = get_predicate(&mut acir_gen.var_cache,binary, evaluator, ctx);
                let (_, r_wit) = constraints::evaluate_udiv(
                    l_c.expression(),
                    r_c.expression(),
                    max_size,
                    predicate.expression(),
                    evaluator,
                );
                InternalVar::from(r_wit)
            }
            BinaryOp::Srem(_) => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                InternalVar::from(
                // TODO: we should use variable naming here instead of .1
                constraints::evaluate_sdiv(l_c.expression(), r_c.expression(), evaluator).1,
            )},
            BinaryOp::Div(_) => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let predicate = get_predicate(&mut acir_gen.var_cache,binary, evaluator, ctx).expression().clone();
                if let Some(r_value) = r_c.to_const() {
                    if r_value.is_zero() {
                        panic!("Panic - division by zero");
                    } else {
                        (l_c.expression() * r_value.inverse()).into()
                    }
                } else {
                    //TODO avoid creating witnesses here.
                    let x_witness = acir_gen.var_cache.get_or_compute_witness(r_c, evaluator).expect("unexpected constant expression"); 
                    let inverse = Expression::from(constraints::evaluate_inverse(
                        x_witness, &predicate, evaluator,
                    ));
                    InternalVar::from(constraints::mul_with_witness(
                        evaluator,
                        l_c.expression(),
                        &inverse,
                    ))
                }
            }
            BinaryOp::Eq => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var(binary.rhs, evaluator, ctx);
                InternalVar::from(
                operations::cmp::evaluate_eq(acir_gen,binary.lhs, binary.rhs, l_c, r_c, ctx, evaluator),
            )},
            BinaryOp::Ne => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var(binary.rhs, evaluator, ctx);
                InternalVar::from(
                operations::cmp::evaluate_neq(acir_gen,binary.lhs, binary.rhs, l_c, r_c, ctx, evaluator),
            )},
            BinaryOp::Ult => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let size = ctx[binary.lhs].get_type().bits();
                constraints::evaluate_cmp(
                    l_c.expression(),
                    r_c.expression(),
                    size,
                    false,
                    evaluator,
                )
                .into()
            }
            BinaryOp::Ule => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let size = ctx[binary.lhs].get_type().bits();
                let e = constraints::evaluate_cmp(
                    r_c.expression(),
                    l_c.expression(),
                    size,
                    false,
                    evaluator,
                );
                constraints::subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Slt => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let s = ctx[binary.lhs].get_type().bits();
                constraints::evaluate_cmp(l_c.expression(), r_c.expression(), s, true, evaluator)
                    .into()
            }
            BinaryOp::Sle => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let s = ctx[binary.lhs].get_type().bits();
                let e = constraints::evaluate_cmp(
                    r_c.expression(),
                    l_c.expression(),
                    s,
                    true,
                    evaluator,
                );
                constraints::subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Lt | BinaryOp::Lte => {
                // TODO Create an issue to change this function to return a RuntimeErrorKind
                // TODO then replace `unimplemented` with an error
                // TODO (This is a breaking change)
                unimplemented!(
                "Field comparison is not implemented yet, try to cast arguments to integer type"
            )
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                let l_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.lhs, evaluator, ctx);
                let r_c = acir_gen.var_cache.get_or_compute_internal_var_unwrap(binary.rhs, evaluator, ctx);
                let bit_size = res_type.bits();
                let opcode = binary.operator.clone();
                let bitwise_result = match operations::bitwise::simplify_bitwise(&l_c, &r_c, bit_size, &opcode) {
                    Some(simplified_internal_var) => simplified_internal_var.expression().clone(),
                    None => operations::bitwise::evaluate_bitwise(l_c, r_c, bit_size, evaluator, &mut acir_gen.var_cache, ctx, opcode),
                };
                InternalVar::from(bitwise_result)
            }
            BinaryOp::Shl | BinaryOp::Shr(_) => todo!("ShiftLeft and ShiftRight operations with shifts which are only known at runtime are not yet implemented."),
            i @ BinaryOp::Assign => unreachable!("Invalid Instruction: {:?}", i),
        };
    Some(binary_output)
}
