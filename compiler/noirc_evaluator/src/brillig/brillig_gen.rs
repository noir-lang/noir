pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_globals;
pub(crate) mod brillig_slice_ops;
mod constant_allocation;
mod variable_liveness;

use acvm::FieldElement;
use noirc_errors::call_stack::CallStack;

use self::brillig_fn::FunctionContext;
use super::{
    brillig_ir::{
        artifact::{BrilligParameter, GeneratedBrillig},
        BrilligContext,
    },
    Brillig, BrilligVariable, ValueId,
};
use crate::{errors::InternalError, ssa::ir::function::Function};

pub(crate) fn gen_brillig_for(
    func: &Function,
    arguments: Vec<BrilligParameter>,
    brillig: &Brillig,
) -> Result<GeneratedBrillig<FieldElement>, InternalError> {
    // Create the entry point artifact
    let mut entry_point = BrilligContext::new_entry_point_artifact(
        arguments,
        FunctionContext::return_values(func),
        func.id(),
        true,
    );
    entry_point.name = func.name().to_string();

    // Link the entry point with all dependencies
    while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
        let artifact = &brillig.find_by_label(unresolved_fn_label.clone());
        let artifact = match artifact {
            Some(artifact) => artifact,
            None => {
                return Err(InternalError::General {
                    message: format!("Cannot find linked fn {unresolved_fn_label}"),
                    call_stack: CallStack::new(),
                })
            }
        };
        entry_point.link_with(artifact);
        // Insert the range of opcode locations occupied by a procedure
        if let Some(procedure_id) = &artifact.procedure {
            let num_opcodes = entry_point.byte_code.len();
            let previous_num_opcodes = entry_point.byte_code.len() - artifact.byte_code.len();
            // We subtract one as to keep the range inclusive on both ends
            entry_point
                .procedure_locations
                .insert(procedure_id.clone(), (previous_num_opcodes, num_opcodes - 1));
        }
    }
    // Generate the final bytecode
    Ok(entry_point.finish())
}
