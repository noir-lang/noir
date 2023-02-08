use crate::{
    ssa::{
        acir_gen::{constraints, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        node::NodeId,
    },
    Evaluator,
};
use acvm::FieldElement;

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
    let sub = constraints::subtract(l_c.expression(), FieldElement::one(), r_c.expression());
    let result = constraints::add(
        &constraints::mul_with_witness(evaluator, cond.expression(), &sub),
        FieldElement::one(),
        r_c.expression(),
    );
    Some(result.into())
}
