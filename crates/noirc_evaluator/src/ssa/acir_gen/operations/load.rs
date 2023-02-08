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
    memory_map: &mut MemoryMap,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    //retrieves the value from the map if address is known at compile time:
    //address = l_c and should be constant
    let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);

    let array_element = match index.to_const() {
        Some(index) => {
            let idx = mem::Memory::as_u32(index);
            let mem_array = &ctx.mem[array_id];

            memory_map
                .load_array_element_constant_index(mem_array, idx)
                .expect("ICE: index {idx} was out of bounds for array of length {mem_array.len}")
        }
        None => unimplemented!("dynamic arrays are not implemented yet"),
    };
    Some(array_element)
}
