use noirc_errors::Location;

use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    ssa::{
        acir_gen::{acir_mem::AcirMem, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        mem::{self, ArrayId},
        node::NodeId,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    array_id: ArrayId,
    index: NodeId,
    memory_map: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    location: Option<Location>,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Result<InternalVar, RuntimeError> {
    //retrieves the value from the map if address is known at compile time:
    //address = l_c and should be constant
    let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);

    match index.to_const() {
        Some(index) => {
            let idx = mem::Memory::as_u32(index);
            let mem_array = &ctx.mem[array_id];

            memory_map.load_array_element_constant_index(mem_array, idx).ok_or(RuntimeError {
                location,
                kind: RuntimeErrorKind::ArrayOutOfBounds {
                    index: idx as u128,
                    bound: mem_array.len as u128,
                },
            })
        }
        None => unimplemented!("dynamic arrays are not implemented yet"),
    }
}
