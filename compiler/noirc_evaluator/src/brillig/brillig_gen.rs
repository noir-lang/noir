pub(crate) mod brillig_black_box;
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_slice_ops;
mod constant_allocation;
mod variable_liveness;

use acvm::FieldElement;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use self::{brillig_block::BrilligBlock, brillig_fn::FunctionContext};
use super::{
    brillig_ir::{
        artifact::{BrilligArtifact, BrilligParameter, GeneratedBrillig, Label},
        BrilligContext,
    }, Brillig, BrilligVariable, DenseMap, Value, ValueId
};
use crate::{
    errors::InternalError,
    ssa::ir::{call_stack::CallStack, function::Function},
};

/// Converting an SSA function into Brillig bytecode.
pub(crate) fn convert_ssa_function(
    func: &Function,
    enable_debug_trace: bool,
    globals: &HashMap<ValueId, BrilligVariable>,
) -> BrilligArtifact<FieldElement> {
    dbg!(func.name());

    let mut brillig_context = BrilligContext::new(enable_debug_trace);

    dbg!(globals.clone());

    let global_values = globals.iter().map(|(value, _)| *value).collect::<HashSet<_>>();

    let mut function_context = FunctionContext::new(func, globals);

    brillig_context.enter_context(Label::function(func.id()));

    brillig_context.call_check_max_stack_depth_procedure();

    // for (id, _) in globals.iter() {
    //     let typ = func.dfg.type_of_value(id);
    //     let value_id = func.dfg.resolve(id);
    //     let variable = allocate_value(value_id, &mut brillig_context, &func.dfg);
    //     dbg!(variable.clone());
    //     if function_context.ssa_value_allocations.insert(value_id, variable).is_some() {
    //         let value = &func.dfg[value_id];
    //         dbg!(value.clone());
    //         unreachable!("ICE: ValueId {value_id:?} was already in cache");
    //     }
    // }
    
    dbg!(function_context.ssa_value_allocations.len());
    dbg!(func.name());
    for block in function_context.blocks.clone() {
        BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg, &global_values);
    }

    let mut artifact = brillig_context.artifact();
    artifact.name = func.name().to_string();
    artifact
}

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
    );
    entry_point.name = func.name().to_string();

    // Link the entry point with all dependencies
    while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
        println!("unresolved_fn_label: {}", unresolved_fn_label);
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
