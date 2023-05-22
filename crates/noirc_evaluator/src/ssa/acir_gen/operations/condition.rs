use crate::{
    ssa::{
        acir_gen::{constraints, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        node::NodeId,
    },
    Evaluator,
};
use acvm::{acir::native_types::Expression, FieldElement};

pub(crate) fn evaluate(
    condition: NodeId,
    lhs: NodeId,
    rhs: NodeId,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    let cond = var_cache.get_or_compute_internal_var_unwrap(condition, evaluator, ctx);
    let l_c = var_cache.get_or_compute_internal_var_unwrap(lhs, evaluator, ctx);
    let r_c = var_cache.get_or_compute_internal_var_unwrap(rhs, evaluator, ctx);
    let result =
        evaluate_expression(cond.expression(), l_c.expression(), r_c.expression(), evaluator);
    Some(result.into())
}

pub(super) fn evaluate_expression(
    condition: &Expression,
    lhs: &Expression,
    rhs: &Expression,
    evaluator: &mut Evaluator,
) -> Expression {
    let sub = constraints::subtract(lhs, FieldElement::one(), rhs);
    constraints::add(
        &constraints::mul_with_witness(evaluator, condition, &sub),
        FieldElement::one(),
        rhs,
    )
}
