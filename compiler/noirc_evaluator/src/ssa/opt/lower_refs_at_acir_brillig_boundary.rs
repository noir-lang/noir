//! This pass materializes immutable reference arguments at the ACIR→Brillig function-call boundary.
//!
//! ## Background
//!
//! When a constrained (ACIR) function passes an immutable reference `&x` to an unconstrained
//! (Brillig) function the frontend lowers `&x` to:
//!
//! ```text
//! v_ref = allocate            // allocate a slot
//! store v_val at v_ref        // write the value into it
//! call brillig_fn(v_ref, …)   // pass the reference
//! ```
//!
//! ACIR functions cannot contain `Allocate`/`Store`/`Load` instructions, so these must be removed
//! before ACIR code-generation.  `mem2reg` cannot remove them because `v_ref` is passed as an
//! argument to the Brillig call (i.e. it "escapes").
//!
//! ## What this pass does
//!
//! For every ACIR function that calls a Brillig function with one or more reference arguments it:
//!
//! 1. Finds the value stored into each reference (the single `Store` that must precede the call
//!    for a fresh `&x` allocation).
//! 2. Creates a thin **wrapper** Brillig function that
//!    a. accepts a plain *value* instead of the reference, and
//!    b. allocates a local slot, stores the value, and calls the original Brillig function with
//!    the resulting reference.
//! 3. Replaces the original call in the ACIR function with a call to the wrapper passing the stored
//!    value directly.
//!
//! After this transformation:
//! - The ACIR function no longer contains `Allocate` or `Store` instructions for the rewritten
//!   references (they become dead code, removed by the DIE pass that follows).
//! - The wrapper Brillig function carries all the necessary memory operations, which are legal
//!   in Brillig.

use std::sync::Arc;

use noirc_frontend::monomorphization::ast::InlineType;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        dfg::GlobalsGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Materialize immutable reference arguments at every ACIR→Brillig call boundary.
    ///
    /// See the module documentation for a detailed explanation.
    pub(crate) fn lower_refs_at_acir_brillig_boundary(mut self) -> Self {
        let transformations = collect_transformations(&self);

        if transformations.is_empty() {
            return self;
        }

        for transform in transformations {
            // Build and register the wrapper Brillig function.
            let wrapper_id = self.add_fn(|wrapper_id| {
                build_wrapper(
                    wrapper_id,
                    transform.callee_id,
                    &transform.callee_name,
                    &transform.callee_param_types,
                    &transform.ref_arg_positions,
                    transform.call_result_types.clone(),
                    transform.callee_globals.clone(),
                )
            });

            // Update the call in the ACIR function.
            let acir_func = self.functions.get_mut(&transform.acir_func_id).unwrap();

            // Register the wrapper as a callable value in the ACIR function's DFG.
            let wrapper_val = acir_func.dfg.import_function(wrapper_id);

            // Build the new argument list: replace reference args with their stored values.
            let new_args: Vec<ValueId> = transform
                .original_args
                .iter()
                .enumerate()
                .map(|(pos, &arg)| {
                    if let Some(idx) = transform.ref_arg_positions.iter().position(|&p| p == pos) {
                        transform.stored_values[idx]
                    } else {
                        arg
                    }
                })
                .collect();

            acir_func.dfg[transform.call_instr_id] =
                Instruction::Call { func: wrapper_val, arguments: new_args };
        }

        self
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// All the data needed to rewrite one call site.
struct Transformation {
    acir_func_id: FunctionId,
    call_instr_id: InstructionId,
    callee_id: FunctionId,
    callee_name: String,
    callee_param_types: Vec<Type>,
    callee_globals: Arc<GlobalsGraph>,
    original_args: Vec<ValueId>,
    /// Positions in `original_args` that are reference-typed.
    ref_arg_positions: Vec<usize>,
    /// `stored_values[i]` is the value stored into `original_args[ref_arg_positions[i]]`.
    stored_values: Vec<ValueId>,
    call_result_types: Vec<Type>,
}

/// Walk every ACIR function, find calls to Brillig functions that pass
/// references, and collect the information needed to rewrite them.
fn collect_transformations(ssa: &Ssa) -> Vec<Transformation> {
    let mut out = Vec::new();

    for (&func_id, func) in &ssa.functions {
        if !func.runtime().is_acir() {
            continue;
        }

        for block_id in func.reachable_blocks() {
            for &instr_id in func.dfg[block_id].instructions() {
                let Instruction::Call { func: callee_val_id, arguments } = &func.dfg[instr_id]
                else {
                    continue;
                };
                let Value::Function(callee_id) = func.dfg[*callee_val_id] else {
                    continue;
                };
                let callee = &ssa.functions[&callee_id];
                if !callee.runtime().is_brillig() {
                    continue;
                }

                let mut ref_arg_positions = Vec::new();
                let mut stored_values = Vec::new();

                for (pos, &arg_id) in arguments.iter().enumerate() {
                    if !matches!(func.dfg.type_of_value(arg_id), Type::Reference(_)) {
                        continue;
                    }
                    if let Some(stored) = find_stored_value(func, arg_id, block_id) {
                        ref_arg_positions.push(pos);
                        stored_values.push(stored);
                    }
                }

                if ref_arg_positions.is_empty() {
                    continue;
                }

                // Collect return types from the instruction results.
                let call_result_types: Vec<Type> = func
                    .dfg
                    .instruction_results(instr_id)
                    .iter()
                    .map(|&v| func.dfg.type_of_value(v))
                    .collect();

                // Collect callee parameter types.
                let callee_entry = callee.entry_block();
                let callee_param_types: Vec<Type> = callee.dfg[callee_entry]
                    .parameters()
                    .iter()
                    .map(|&p| callee.dfg.type_of_value(p))
                    .collect();

                out.push(Transformation {
                    acir_func_id: func_id,
                    call_instr_id: instr_id,
                    callee_id,
                    callee_name: callee.name().to_string(),
                    callee_param_types,
                    callee_globals: callee.dfg.globals.clone(),
                    original_args: arguments.clone(),
                    ref_arg_positions,
                    stored_values,
                    call_result_types,
                });
            }
        }
    }

    out
}

/// Search `block_id` in `func` for a `Store { address: ref_val, value: v }`
/// instruction and return `v`.
fn find_stored_value(func: &Function, ref_val: ValueId, block_id: BasicBlockId) -> Option<ValueId> {
    for &instr_id in func.dfg[block_id].instructions() {
        if let Instruction::Store { address, value } = func.dfg[instr_id]
            && address == ref_val
        {
            return Some(value);
        }
    }
    None
}

/// Build a thin Brillig wrapper function:
/// - Accepts plain values in place of reference parameters.
/// - Allocates a local slot for each reference parameter, stores the value.
/// - Calls the original Brillig function with the resulting references.
fn build_wrapper(
    wrapper_id: FunctionId,
    callee_id: FunctionId,
    callee_name: &str,
    callee_param_types: &[Type],
    ref_arg_positions: &[usize],
    call_result_types: Vec<Type>,
    callee_globals: Arc<GlobalsGraph>,
) -> Function {
    let mut builder = FunctionBuilder::new(format!("{callee_name}__ref_wrapper"), wrapper_id);
    builder.set_runtime(RuntimeType::Brillig(InlineType::Inline));
    builder.set_globals(callee_globals);

    // Add one parameter per callee parameter.
    // For reference positions: use the inner (pointed-to) type.
    let params: Vec<ValueId> = callee_param_types
        .iter()
        .enumerate()
        .map(|(pos, typ)| {
            let param_type = if ref_arg_positions.contains(&pos) {
                match typ {
                    Type::Reference(inner) => (**inner).clone(),
                    _ => typ.clone(),
                }
            } else {
                typ.clone()
            };
            builder.add_parameter(param_type)
        })
        .collect();

    // Build the argument list for the inner call.
    let mut inner_args: Vec<ValueId> = Vec::with_capacity(params.len());
    for (pos, &param_val) in params.iter().enumerate() {
        if ref_arg_positions.contains(&pos) {
            let inner_type = match &callee_param_types[pos] {
                Type::Reference(inner) => (**inner).clone(),
                other => other.clone(),
            };
            let alloc = builder.insert_allocate(inner_type);
            builder.insert_store(alloc, param_val);
            inner_args.push(alloc);
        } else {
            inner_args.push(param_val);
        }
    }

    let callee_val = builder.import_function(callee_id);
    let results = builder.insert_call(callee_val, inner_args, call_result_types).to_vec();
    builder.terminate_with_return(results);

    builder.current_function
}
