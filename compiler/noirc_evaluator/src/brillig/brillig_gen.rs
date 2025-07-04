//! The code generation logic for converting [crate::ssa] objects into their respective [Brillig] artifacts.  
pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_globals;
pub(crate) mod brillig_slice_ops;
pub(crate) mod constant_allocation;
mod variable_liveness;

use acvm::FieldElement;
use noirc_errors::call_stack::CallStack;

use self::brillig_fn::FunctionContext;
use super::{
    Brillig, BrilligOptions, BrilligVariable, ValueId,
    brillig_ir::{
        BrilligContext,
        artifact::{BrilligParameter, GeneratedBrillig},
    },
};
use crate::{errors::InternalError, ssa::ir::function::Function};

/// Generates a complete Brillig entry point artifact for a given SSA-level [Function], linking all dependencies.
///
/// This function is responsible for generating a final Brillig artifact corresponding to a compiled SSA [Function].
/// It sets up the entry point context, registers input/output parameters, and recursively resolves and links
/// all transitive Brillig function dependencies.
///
/// # Parameters
/// - func: The SSA [Function] to compile as the entry point.
/// - arguments: Brillig-compatible [BrilligParameter] inputs to the function
/// - brillig: The [context structure][Brillig] of all known Brillig artifacts for dependency resolution.
/// - options: Brillig compilation options (e.g., debug trace settings).
///
/// # Returns
/// - Ok([GeneratedBrillig]): Fully linked artifact for the entry point that can be executed as a Brillig program.
/// - Err([InternalError]): If linking fails to find a dependency
///
/// # Panics
/// - If the global memory size for the function has not been precomputed.
pub(crate) fn gen_brillig_for(
    func: &Function,
    arguments: Vec<BrilligParameter>,
    brillig: &Brillig,
    options: &BrilligOptions,
) -> Result<GeneratedBrillig<FieldElement>, InternalError> {
    // Create the entry point artifact
    let globals_memory_size = brillig
        .globals_memory_size
        .get(&func.id())
        .copied()
        .expect("Should have the globals memory size specified for an entry point");

    let options = BrilligOptions { enable_debug_trace: false, ..*options };

    let mut entry_point = BrilligContext::new_entry_point_artifact(
        arguments,
        FunctionContext::return_values(func),
        func.id(),
        true,
        globals_memory_size,
        &options,
    );
    entry_point.name = func.name().to_string();

    // Link the entry point with all dependencies
    while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
        let artifact = &brillig.find_by_label(unresolved_fn_label.clone(), &options);
        let artifact = match artifact {
            Some(artifact) => artifact,
            None => {
                return Err(InternalError::General {
                    message: format!("Cannot find linked fn {unresolved_fn_label}"),
                    call_stack: CallStack::new(),
                });
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
