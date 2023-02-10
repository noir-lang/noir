use crate::{
    ssa::{
        acir_gen::{internal_var_cache::InternalVarCache, memory_map::MemoryMap, InternalVar},
        context::SsaContext,
        mem::{self, ArrayId},
        node::NodeId,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    array_id: ArrayId,
    index: NodeId,
    value: NodeId,
    memory_map: &mut MemoryMap,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    //maps the address to the rhs if address is known at compile time
    let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);
    let value = var_cache.get_or_compute_internal_var_unwrap(value, evaluator, ctx);

    match index.to_const() {
        Some(index) => {
            let idx = mem::Memory::as_u32(index);
            let absolute_adr = ctx.mem[array_id].absolute_adr(idx);
            memory_map.insert(absolute_adr, value);
            //we do not generate constraint, so no output.
            None
        }
        None => todo!("dynamic arrays are not implemented yet"),
    }
}
