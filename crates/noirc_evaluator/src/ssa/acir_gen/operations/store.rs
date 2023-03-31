use acvm::acir::native_types::Expression;

use crate::{
    errors::RuntimeError,
    ssa::{
        acir_gen::{
            acir_mem::AcirMem,
            internal_var_cache::InternalVarCache,
            operations::{condition, load},
            InternalVar,
        },
        context::SsaContext,
        mem,
        node::Operation,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    store: &Operation,
    acir_mem: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Result<Option<InternalVar>, RuntimeError> {
    if let Operation::Store { array_id, index, value, predicate, .. } = *store {
        //maps the address to the rhs if address is known at compile time
        let index_var = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);
        let value_var = var_cache.get_or_compute_internal_var_unwrap(value, evaluator, ctx);
        let value_with_predicate = if let Some(predicate) = predicate {
            if predicate.is_dummy() || ctx.is_one(predicate) {
                value_var
            } else if ctx.is_zero(predicate) {
                return Ok(None);
            } else {
                let pred = var_cache.get_or_compute_internal_var_unwrap(predicate, evaluator, ctx);
                let dummy_load =
                    load::evaluate(array_id, index, acir_mem, var_cache, None, evaluator, ctx)?;
                let result = condition::evaluate_expression(
                    pred.expression(),
                    value_var.expression(),
                    dummy_load.expression(),
                    evaluator,
                );
                result.into()
            }
        } else {
            value_var
        };

        match index_var.to_const() {
            Some(idx) => {
                let idx = mem::Memory::as_u32(idx);
                acir_mem.insert(array_id, idx, value_with_predicate);
            }
            None => {
                acir_mem.add_to_trace(
                    &array_id,
                    index_var.to_expression(),
                    value_with_predicate.to_expression(),
                    Expression::one(),
                );
            }
        }
    } else {
        unreachable!("Expected store, got {:?}", store.opcode());
    }
    //we do not generate constraint, so no output.
    Ok(None)
}
