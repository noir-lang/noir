use crate::{
    ssa::{
        acir_gen::{constraints, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        node::NodeId,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    value: &NodeId,
    bit_size: u32,
    max_bit_size: u32,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    let value = var_cache.get_or_compute_internal_var_unwrap(*value, evaluator, ctx);
    Some(InternalVar::from_expression(constraints::evaluate_truncate(
        value.expression(),
        bit_size,
        max_bit_size,
        evaluator,
    )))
}
