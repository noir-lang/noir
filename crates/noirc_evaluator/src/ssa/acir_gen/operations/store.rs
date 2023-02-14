use crate::{
    ssa::{
        acir_gen::{acir_mem::AcirMem, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        mem::{self},
        node::Operation,
    },
    Evaluator,
};

use super::condition;

pub(crate) fn evaluate(
    store: &Operation,
    acir_mem: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    if let Operation::Store { array_id, index, value, predicate } = *store {
        //maps the address to the rhs if address is known at compile time
        let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);
        let value = var_cache.get_or_compute_internal_var_unwrap(value, evaluator, ctx);

        match index.to_const() {
            Some(index) => {
                let idx = mem::Memory::as_u32(index);
                let value_with_predicate = if let Some(predicate) = predicate {
                    if predicate.is_dummy() || ctx.is_one(predicate) {
                        value
                    } else if ctx.is_zero(predicate) {
                        return None;
                    } else {
                        let pred =
                            var_cache.get_or_compute_internal_var_unwrap(predicate, evaluator, ctx);
                        let dummy_load = acir_mem
                            .load_array_element_constant_index(&ctx.mem[array_id], idx)
                            .unwrap();
                        let result = condition::evaluate_expression(
                            pred.expression(),
                            value.expression(),
                            dummy_load.expression(),
                            evaluator,
                        );
                        result.into()
                    }
                } else {
                    value
                };
                acir_mem.insert(array_id, idx, value_with_predicate);
                //we do not generate constraint, so no output.
                None
            }
            None => todo!("dynamic arrays are not implemented yet"),
        }
    } else {
        unreachable!("Expected store, got {:?}", store.opcode());
    }
}
