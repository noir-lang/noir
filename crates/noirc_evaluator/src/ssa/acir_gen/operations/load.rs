use acvm::acir::native_types::Expression;

use crate::{
    ssa::{
        acir_gen::{
            acir_mem::AcirMem, expression_from_witness, internal_var_cache::InternalVarCache,
            InternalVar,
        },
        context::SsaContext,
        mem::{self, ArrayId},
        node::NodeId,
    },
    Evaluator,
};

// Returns a variable corresponding to the element at the provided index in the array
// Returns None if index is constant and out-of-bound.
pub(crate) fn evaluate(
    array_id: ArrayId,
    index: NodeId,
    acir_mem: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    let mem_array = &ctx.mem[array_id];
    let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);

    if let Some(idx) = index.to_const() {
        let idx = mem::Memory::as_u32(idx);
        // Check to see if the index has gone out of bounds
        let array_length = mem_array.len;
        if idx >= array_length {
            return None; // IndexOutOfBoundsError
        }

        let array_element = acir_mem.load_array_element_constant_index(array_id, idx);
        if array_element.is_some() {
            return array_element;
        }
    }

    let w = evaluator.add_witness_to_cs();
    acir_mem.add_to_trace(
        &array_id,
        index.to_expression(),
        expression_from_witness(w),
        Expression::zero(),
    );
    Some(InternalVar::from_witness(w))
}
