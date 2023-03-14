use acvm::{
    acir::circuit::opcodes::Opcode as AcirOpcode, acir::native_types::Expression, FieldElement,
};

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
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    let value = var_cache.get_or_compute_internal_var_unwrap(*value, evaluator, ctx);
    let subtract =
        constraints::subtract(&Expression::one(), FieldElement::one(), value.expression());
    evaluator.push_opcode(AcirOpcode::Arithmetic(subtract));
    Some(value)
}
