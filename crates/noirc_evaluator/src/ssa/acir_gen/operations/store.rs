use acvm::FieldElement;

use crate::{
    ssa::{
        acir_gen::{
            constraints::{add, mul_with_witness, subtract},
            internal_var_cache::InternalVarCache,
            memory_map::MemoryMap,
            InternalVar,
        },
        context::SsaContext,
        mem::{self},
        node::Operation,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    store: &Operation,
    memory_map: &mut MemoryMap,
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
                let absolute_adr = ctx.mem[array_id].absolute_adr(idx);
                let value_with_predicate = if let Some(predicate) = predicate {
                    if predicate.is_dummy() || ctx.is_one(predicate) {
                        value
                    } else if ctx.is_zero(predicate) {
                        return None;
                    } else {
                        let pred =
                            var_cache.get_or_compute_internal_var_unwrap(predicate, evaluator, ctx);
                        let dummy = memory_map
                            .load_array_element_constant_index(&ctx.mem[array_id], idx)
                            .unwrap();
                        let x = mul_with_witness(evaluator, pred.expression(), value.expression());
                        let y = subtract(
                            dummy.expression(),
                            FieldElement::one(),
                            &mul_with_witness(evaluator, pred.expression(), dummy.expression()),
                        );
                        InternalVar::from(add(&x, FieldElement::one(), &y))
                    }
                } else {
                    value
                };

                memory_map.insert(absolute_adr, value_with_predicate);
                //we do not generate constraint, so no output.
                None
            }
            None => todo!("dynamic arrays are not implemented yet"),
        }
    } else {
        unreachable!("Expected store, got {:?}", store.opcode());
    }
}
