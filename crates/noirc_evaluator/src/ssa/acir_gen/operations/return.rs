use crate::{
    errors::RuntimeErrorKind,
    ssa::{
        acir_gen::{internal_var_cache::InternalVarCache, memory_map::MemoryMap, InternalVar},
        context::SsaContext,
        mem::Memory,
        node::NodeId,
    },
    Evaluator,
};

pub(crate) fn evaluate_return_op(
    node_ids: &[NodeId],
    memory_map: &mut MemoryMap,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Result<Option<InternalVar>, RuntimeErrorKind> {
    // XXX: When we return a node_id that was created from
    // the UnitType, there is a witness associated with it
    // Ideally no witnesses are created for such types.

    // This can only ever be called in the main context.
    // In all other context's, the return operation is transformed.

    for node_id in node_ids {
        // An array produces a single node_id
        // We therefore need to check if the node_id is referring to an array
        // and deference to get the elements
        let objects = match Memory::deref(ctx, *node_id) {
            Some(a) => {
                let array = &ctx.mem[a];
                memory_map.load_array(array)
            }
            None => {
                vec![var_cache.get_or_compute_internal_var_unwrap(*node_id, evaluator, ctx)]
            }
        };

        for mut object in objects {
            let witness = object.get_or_compute_witness(evaluator, true).expect(
                "infallible: `None` can only be returned when we disallow constant Expressions.",
            );
            // Before pushing to the public inputs, we need to check that
            // it was not a private ABI input
            if evaluator.is_private_abi_input(witness) {
                return Err(RuntimeErrorKind::Spanless(String::from(
                    "we do not allow private ABI inputs to be returned as public outputs",
                )));
            }
            evaluator.public_inputs.push(witness);
        }
    }

    Ok(None)
}
