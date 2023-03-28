use acvm::acir::native_types::Witness;
use noirc_abi::MAIN_RETURN_NAME;

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
                memory_map.load_array(array)
            }
            None => {
                vec![var_cache.get_or_compute_internal_var_unwrap(*node_id, evaluator, ctx)]
            }
        };

        let mut witnesses: Vec<Witness> = Vec::new();
        for object in objects {
            let witness = var_cache.get_or_compute_witness_unwrap(object, evaluator, ctx);
            // Before pushing to the public inputs, we need to check that
            // it was not a private ABI input
            if evaluator.is_private_abi_input(witness) {
                return Err(RuntimeErrorKind::Spanless(String::from(
                    "we do not allow private ABI inputs to be returned as public outputs",
                )));
            }
            witnesses.push(witness);
        }
        evaluator.public_inputs.extend(witnesses.clone());
        evaluator.param_witnesses.entry(MAIN_RETURN_NAME.to_owned())
            .or_default()
            .append(&mut witnesses);
    }

    Ok(None)
}
