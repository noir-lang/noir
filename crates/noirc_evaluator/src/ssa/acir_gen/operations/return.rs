use acvm::acir::native_types::Expression;

use crate::{
    errors::RuntimeErrorKind,
    ssa::{
        acir_gen::{acir_mem::AcirMem, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        mem::Memory,
        node::NodeId,
    },
    Evaluator,
};

pub(crate) fn evaluate(
    node_ids: &[NodeId],
    memory_map: &mut AcirMem,
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
                memory_map.return_array(a);
                memory_map.load_array(array, evaluator)
            }
            None => {
                vec![var_cache.get_or_compute_internal_var_unwrap(*node_id, evaluator, ctx)]
            }
        };

        for object in objects {
            let witness = var_cache.get_or_compute_witness_unwrap(object, evaluator, ctx);
            // Before pushing to the public inputs, we need to check that
            // it was not a private ABI input
            if evaluator.is_private_abi_input(witness) {
                return Err(RuntimeErrorKind::PrivateAbiInput);
            }
            // Check if the outputted witness needs separating from an existing occurrence in the
            // abi. This behavior stems from usage of the `distinct` keyword.
            let return_witness = if evaluator.should_proxy_witness_for_abi_output(witness) {
                let proxy_constraint = Expression::from(witness);
                evaluator.create_intermediate_variable(proxy_constraint)
            } else {
                witness
            };
            evaluator.return_values.push(return_witness);
        }
    }

    Ok(None)
}
