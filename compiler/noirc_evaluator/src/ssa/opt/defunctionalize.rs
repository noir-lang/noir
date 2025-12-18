//! This module defines the defunctionalization pass for the SSA IR.
//! Certain IR targets (e.g., Brillig and ACIR) do not support higher-order functions directly.
//!
//! The pass eliminates higher-order functions (a function which accepts function values as arguments or returns functions)
//! by transforming functions used as values (i.e., first-class functions)
//! into constant numbers (fields) that represent their function IDs.
//!
//! Defunctionalization handles higher-order functions by lowering function values into
//! constant identifiers and replacing calls of function values with calls to a single
//! dispatch `apply` function.
//!
//! ## How the pass works:
//! - Every function used as a value (e.g., passed as a parameter) is assigned a unique [NumericType::NativeField] value.
//!   This value now represents the first-class function's ID.
//! - All call instructions with non-literal targets are replaced by calls to an `apply` function.
//! - The `apply` function is a dispatcher. It takes the function ID as its first argument
//!   and calls the appropriate function based on that ID.
//!
//! Pseudocode of an `apply` function is given below:
//! ```text
//! fn apply(function_id: Field, arg1: Field, arg2: Field) -> Field {
//!     match function_id {
//!         0 -> function0(arg1, arg2),
//!         1 -> function1(arg1, arg2),
//!         ...
//!         N -> functionN(arg1, arg2),
//!     }
//! }
//! ```
//!
//! After this pass all first-class functions are replaced with numeric IDs
//! and calls are routed via the newly generated `apply` functions.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::InlineType;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType, Signature},
        instruction::{BinaryOp, Instruction},
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    opt::pure::Purity,
    ssa_gen::Ssa,
};
use rustc_hash::FxHashMap as HashMap;

/// Represents an 'apply' function created by this pass to dispatch higher order functions to.
/// Pseudocode of an `apply` function is given below:
/// ```text
/// fn apply(function_id: Field, arg1: Field, arg2: Field) -> Field {
///     match function_id {
///         0 -> function0(arg1, arg2),
///         1 -> function1(arg1, arg2),
///         ...
///         N -> functionN(arg1, arg2),
///     }
/// }
/// ```
/// Apply functions generally take the function to apply as their first parameter. This is a Field value
/// obtained by converting the FunctionId into a Field. The remaining parameters of apply are the
/// arguments to forward to this function when calling it internally.
#[derive(Debug, Clone, Copy)]
struct ApplyFunction {
    id: FunctionId,
    dispatches_to_multiple_functions: bool,
}

/// All functions used as a value that share the same signature and runtime type
/// Maps ([Signature], Caller [RuntimeType]) -> Vec<([FunctionId], Callee [RuntimeType])>
type Variants = BTreeMap<(Signature, RuntimeType), Vec<(FunctionId, RuntimeType)>>;
/// All generated apply functions for each grouping of function variants.
/// Each apply function is handles a specific ([Signature], [RuntimeType]) group.
/// Maps ([Signature], [RuntimeType]) -> [ApplyFunction]
type ApplyFunctions = HashMap<(Signature, RuntimeType), ApplyFunction>;

/// Performs defunctionalization on all functions
/// This is done by changing all functions as value to be a number (FieldElement)
/// And creating apply functions that dispatch to the correct target by runtime comparisons with constants
#[derive(Debug, Clone)]
struct DefunctionalizationContext {
    apply_functions: ApplyFunctions,
}

impl Ssa {
    /// See [`defunctionalize`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn defunctionalize(mut self) -> Ssa {
        // Check that we have removed all cases we don't handle in this pass.
        #[cfg(debug_assertions)]
        self.functions.values().for_each(defunctionalize_pre_check);

        // Find all functions used as value that share the same signature and runtime type
        let variants = find_variants(&self);

        // Generate the apply functions for the provided variants
        let (apply_functions, purities) = create_apply_functions(&mut self, variants);

        // Setup the pass context
        let context = DefunctionalizationContext { apply_functions };

        // Run defunctionalization over all functions in the SSA
        context.defunctionalize_all(&mut self);

        let purities = Arc::new(purities);
        for function in self.functions.values_mut() {
            function.dfg.set_function_purities(purities.clone());
        }

        // Check that we have established the properties expected from this pass.
        #[cfg(debug_assertions)]
        self.functions.values().for_each(defunctionalize_post_check);

        self
    }
}

impl DefunctionalizationContext {
    /// Defunctionalize all functions in the SSA
    fn defunctionalize_all(mut self, ssa: &mut Ssa) {
        for function in ssa.functions.values_mut() {
            // We mutate value types in `defunctionalize`, so to prevent that from affecting which
            // apply functions are chosen we replace all first-class function calls with calls to
            // the appropriate apply function beforehand.
            self.replace_first_class_calls_with_apply_function(function);

            // Replace any first-class function values with field values. This will also mutate the
            // type of some values, such as block arguments
            self.defunctionalize(function);
        }
    }

    /// Replaces any function calls using first-class function values with calls to the
    /// appropriate `apply` function. Note that this must be done before types are mutated
    /// in `defunctionalize` since this uses the pre-mutated types to query apply functions.
    fn replace_first_class_calls_with_apply_function(&mut self, func: &mut Function) {
        for block_id in func.reachable_blocks() {
            let block = &mut func.dfg[block_id];

            #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
            for instruction_id in block.instructions().to_vec() {
                let instruction = &func.dfg[instruction_id];

                // Operate on call instructions
                let (target_func_id, arguments) = match &instruction {
                    Instruction::Call { func: target_func_id, arguments } => {
                        (*target_func_id, arguments)
                    }
                    _ => continue,
                };

                // If the target is a function used as value
                use Value::Param;
                if matches!(&func.dfg[target_func_id], Param { .. } | Value::Instruction { .. }) {
                    let mut arguments = arguments.clone();
                    let results = func.dfg.instruction_results(instruction_id);
                    let signature = Signature {
                        params: vecmap(&arguments, |param| func.dfg.type_of_value(*param)),
                        returns: vecmap(results, |result| func.dfg.type_of_value(*result)),
                    };

                    // Find the correct apply function
                    let Some(apply_function) = self.get_apply_function(signature, func.runtime())
                    else {
                        // We should have generated an apply function for all function's used as a value,
                        // even if there are no variants for that function call.
                        panic!(
                            "ICE: It is expected to have an apply function for every function used as a value"
                        );
                    };

                    // Replace the instruction with a call to apply
                    let apply_function_value_id = func.dfg.import_function(apply_function.id);
                    if apply_function.dispatches_to_multiple_functions {
                        arguments.insert(0, target_func_id);
                    }
                    let func_id = apply_function_value_id;
                    let replacement_instruction = Instruction::Call { func: func_id, arguments };
                    func.dfg[instruction_id] = replacement_instruction;
                }
            }
        }
    }

    /// Defunctionalize a single function
    fn defunctionalize(&mut self, func: &mut Function) {
        for block_id in func.reachable_blocks() {
            let block = &mut func.dfg[block_id];

            // Temporarily take the parameters here just to avoid cloning them
            let parameters = block.take_parameters();
            for parameter in &parameters {
                let typ = &func.dfg.type_of_value(*parameter);
                if let Some(rep) = replacement_type(typ) {
                    func.dfg.set_type_of_value(*parameter, rep);
                }
            }

            let block = &mut func.dfg[block_id];
            block.set_parameters(parameters);

            // Do the same for the terminator
            let mut terminator = block.take_terminator();
            terminator.map_values_mut(|value| map_function_to_field(func, value).unwrap_or(value));

            let block = &mut func.dfg[block_id];
            block.set_terminator(terminator);

            // Now we can finally change each instruction, replacing
            // each first class function with a field value.
            #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
            for instruction_id in block.instructions().to_vec() {
                let mut instruction = func.dfg[instruction_id].clone();

                if remove_first_class_functions_in_instruction(func, &mut instruction) {
                    func.dfg[instruction_id] = instruction;
                }

                #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
                for result in func.dfg.instruction_results(instruction_id).to_vec() {
                    let typ = &func.dfg.type_of_value(result);
                    if let Some(rep) = replacement_type(typ) {
                        func.dfg.set_type_of_value(result, rep);
                    }
                }
            }
        }
    }

    /// Returns the apply function for the given signature
    fn get_apply_function(
        &self,
        signature: Signature,
        runtime: RuntimeType,
    ) -> Option<ApplyFunction> {
        self.apply_functions.get(&(signature, runtime)).copied()
    }
}

/// Replace any first class functions used in an instruction with a field value.
/// This applies to any function used anywhere else other than the function position
/// of a call instruction. Returns true if the instruction was modified
fn remove_first_class_functions_in_instruction(
    func: &mut Function,
    instruction: &mut Instruction,
) -> bool {
    let mut modified = false;
    let mut map_value = |value: ValueId| {
        if let Some(new_value) = map_function_to_field(func, value) {
            modified = true;
            new_value
        } else {
            value
        }
    };

    if let Instruction::Call { func: _, arguments } = instruction {
        for arg in arguments {
            *arg = map_value(*arg);
        }
    } else if let Instruction::MakeArray { typ, .. } = instruction {
        let mut modified_type = false;
        if let Some(rep) = replacement_type(typ) {
            *typ = rep;
            modified_type = true;
        }

        instruction.map_values_mut(map_value);

        if modified_type {
            modified = true;
        }
    } else {
        instruction.map_values_mut(map_value);
    }

    modified
}

/// Try to map the given function literal to a field, returning Some(field) on success.
/// Returns none if the given value was not a function or doesn't need to be mapped.
fn map_function_to_field(func: &mut Function, value: ValueId) -> Option<ValueId> {
    let typ = func.dfg[value].get_type();
    if let Some(rep) = replacement_type(typ.as_ref()) {
        match &func.dfg[value] {
            // If the value is a static function, transform it to the function id
            Value::Function(id) => {
                let new_value = function_id_to_field(*id);
                return Some(func.dfg.make_constant(new_value, NumericType::NativeField));
            }
            // If the value is a function used as value, just change the type of it
            Value::Instruction { .. } | Value::Param { .. } => {
                func.dfg.set_type_of_value(value, rep);
            }
            _ => (),
        }
    }
    None
}

/// Collects all functions used as values that can be called by their signatures
///
/// Groups all [FunctionId]s used as values by their [Signature] and [RuntimeType],
/// producing a mapping from these tuples to the vector of variant functions to be dynamically dispatched.
///
/// # Arguments
/// - `ssa`: The full [Ssa] structure
///
/// # Returns
/// [Variants] that should then be used to generate apply functions for dispatching
fn find_variants(ssa: &Ssa) -> Variants {
    let mut dynamic_dispatches: BTreeSet<(Signature, RuntimeType)> = BTreeSet::new();
    let mut functions_as_values: BTreeSet<FunctionId> = BTreeSet::new();

    for function in ssa.functions.values() {
        functions_as_values.extend(find_functions_as_values(function));
        dynamic_dispatches.extend(
            find_dynamic_dispatches(function).into_iter().map(|sig| (sig, function.runtime())),
        );
    }

    // Group function variant candidates by their signature
    let mut signature_to_functions_as_value: BTreeMap<Signature, Vec<FunctionId>> = BTreeMap::new();

    for function_id in functions_as_values {
        let signature = ssa.functions[&function_id].signature();
        signature_to_functions_as_value.entry(signature).or_default().push(function_id);
    }

    let mut variants: Variants = BTreeMap::new();

    // Further group function variant candidates by their caller runtime.
    // The target functions are never tested against their runtime, which is assumed to be correct,
    // but there is nothing which guarantees this.
    // Filtering the functions by their runtime at this stage will not work because if the variant
    // is empty, the apply function will create a dummy function. So instead of calling a function with a wrong runtime,
    // we would end-up calling a dummy function.
    // The solution is to add the runtime information to the variants map
    // and defer to create_apply_function for handling the runtime checks.
    for (dispatch_signature, caller_runtime) in dynamic_dispatches {
        let target_fns = signature_to_functions_as_value
            .get(&dispatch_signature)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|f| (f, ssa.functions[&f].runtime()));
        let target_fns: Vec<(FunctionId, RuntimeType)> = target_fns.collect();
        variants.insert((dispatch_signature, caller_runtime), target_fns);
    }

    // We will now have fully constructed our variants map and can return it
    variants
}

/// Finds all literal functions used as values in the given function
fn find_functions_as_values(func: &Function) -> BTreeSet<FunctionId> {
    let mut functions_as_values: BTreeSet<FunctionId> = BTreeSet::new();

    visit_values_other_than_call_target(func, |value| {
        if let Value::Function(id) = value {
            functions_as_values.insert(*id);
        }
    });

    functions_as_values
}

/// Visit all values which are *not* targets of a `Call`.
fn visit_values_other_than_call_target(func: &Function, mut f: impl FnMut(&Value)) {
    let mut process_value = |value_id: ValueId| {
        f(&func.dfg[value_id]);
    };

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];

            // Handle call instructions separately. Functions used in their function field
            // don't have to be first-class values.
            if let Instruction::Call { arguments, .. } = instruction {
                arguments.iter().for_each(|value_id| process_value(*value_id));
            } else {
                instruction.for_each_value(&mut process_value);
            }
        }

        block.unwrap_terminator().for_each_value(&mut process_value);
    }
}

/// Finds all dynamic dispatch signatures in the given function.
/// Note that these are the signatures before function types are mutated to turn into field types.
///
/// A dynamic dispatch is defined as a call into a function value where that
/// value comes from a parameter (i.e., calling a function passed as a function parameter
/// or another instruction (i.e., calling a function returned from another function call).
fn find_dynamic_dispatches(func: &Function) -> BTreeSet<Signature> {
    let mut dispatches = BTreeSet::new();

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];
            match instruction {
                Instruction::Call { func: target, arguments } => {
                    if let Value::Param { .. } | Value::Instruction { .. } = &func.dfg[*target] {
                        let results = func.dfg.instruction_results(*instruction_id);
                        dispatches.insert(Signature {
                            params: vecmap(arguments, |param| func.dfg.type_of_value(*param)),
                            returns: vecmap(results, |result| func.dfg.type_of_value(*result)),
                        });
                    }
                }
                _ => continue,
            };
        }
    }
    dispatches
}

/// Creates all apply functions needed for dispatch of function values.
///
/// This function maintains the grouping set in [Variants], meaning an apply
/// function is grouped by functions that share a signature and runtime.
/// An apply function is only created if there are multiple function variants
/// for a specific ([Signature], [RuntimeType]) group.
/// Otherwise, if there is a single variant that function is simply reused.
///
/// If there are no variants a dummy function is created.
/// A dummy function acts as a safe no-op to continue compilation even though there are no variants
/// for a first-class function call. For more information you can reference [create_dummy_function].
///
/// # Arguments
/// - `ssa`: A mutable reference to the full [Ssa] structure containing all functions.
/// - `variants_map`:  [Variants]
///
/// # Returns
/// - [ApplyFunctions] keyed by each function's signature _before_ functions are changed
///   into field types. The inner apply function itself will have its defunctionalized type,
///   with function values represented as field values.
/// - [HashMap<FunctionId, Purity>] with purities that must be set to all functions in the SSA,
///   as this function might have created dummy pure functions.
fn create_apply_functions(
    ssa: &mut Ssa,
    variants_map: Variants,
) -> (ApplyFunctions, HashMap<FunctionId, Purity>) {
    let mut apply_functions = HashMap::default();
    let mut purities = if ssa.functions.is_empty() {
        HashMap::default()
    } else {
        (*ssa.functions.iter().next().unwrap().1.dfg.function_purities).clone()
    };

    for ((signature, caller_runtime), variants) in variants_map.into_iter() {
        // Calling an ACIR function from a Brillig runtime is not allowed.
        // We expect all ACIR functions called from Brillig to be specialized
        // as Brillig functions at compile time (e.g., before SSA generation).
        // Closures are expected to be duplicated by their runtime.
        // It is expected that the frontend has appropriately extracted the correct closure
        // based on the current runtime of the caller function.
        // However, during defunctionalization we are unable to tell that the correct target is being called so all
        // variants with different runtimes will be included in the apply function.
        // Including a closure variant with a different runtime in the closure variant will cause an ill-formed SSA.
        // Thus, to avoid a broken SSA we simply do not include calls to variants which differ from their callers.
        // It is expected that if the new ID associated with the bad variant is used, that we will still fail as the last
        // function to dispatch constrains that we have an expected ID.
        let pre_runtime_filter_len = variants.len();
        let variants: Vec<(FunctionId, RuntimeType)> = variants
            .into_iter()
            .filter(|(_, runtime)| !(runtime.is_acir() && caller_runtime.is_brillig()))
            .collect();

        let dispatches_to_multiple_functions = variants.len() > 1;

        // This will be the same signature but with each function type replaced with
        // a Field type.
        let mut defunctionalized_signature = signature.clone();

        // Update the shared function signature of the higher-order function variants
        // to replace any function passed as a value to a numeric field type.
        for typ in defunctionalized_signature
            .params
            .iter_mut()
            .chain(&mut defunctionalized_signature.returns)
        {
            if let Some(rep) = replacement_type(typ) {
                *typ = rep;
            }
        }

        let id = if dispatches_to_multiple_functions {
            // If we have multiple variants for this signature and runtime type group
            // we need to generate an apply function.
            create_apply_function(ssa, defunctionalized_signature, caller_runtime, variants)
        } else if !variants.is_empty() {
            // If there is only variant, we can use it directly rather than creating a new apply function.
            variants[0].0
        } else if pre_runtime_filter_len != 0 && caller_runtime.is_brillig() {
            // We had variants, but they were all filtered out.
            // Frontend bug: only ACIR variants in a Brillig group.
            panic!("ICE: invalid defunctionalization: only ACIR variants for a Brillig runtime");
        } else {
            // If no variants exist for a dynamic call we leave removing those dead calls and parameters to DIE.
            // However, we have to construct a dummy function for these dead calls as to keep a well formed SSA
            // and to not break the semantics of other SSA passes before DIE is reached.
            create_dummy_function(ssa, defunctionalized_signature, caller_runtime, &mut purities)
        };
        apply_functions.insert(
            (signature, caller_runtime),
            ApplyFunction { id, dispatches_to_multiple_functions },
        );
    }

    (apply_functions, purities)
}

/// Transforms a [FunctionId] into a [FieldElement]
fn function_id_to_field(function_id: FunctionId) -> FieldElement {
    u128::from(function_id.to_u32()).into()
}

/// Creates a single apply function to enable dispatch across multiple function variants
/// that share the same [Signature] and [RuntimeType].
///
/// This function is responsible for generating an entry point that dispatches between several
/// concrete functions at runtime based on a target field value. It builds a sequence of
/// conditional checks (if-else chain) to compare the target against each
/// function's ID, and calls the matching function.
///
/// These apply functions are to be aggressively inlined as it is assumed that they will be optimized
/// away by the constants at the call site.
///
/// # Arguments
/// - `ssa`: A mutable reference to the full [Ssa] structure containing all functions.
/// - `signature`: The shared [Signature] of all variants but with each `Type::Function` replaced with a field type.
/// - `caller_runtime`: The runtime in which the apply function will be called, used to update inlining policies.
/// - `function_ids`: A non-empty vector of [FunctionId]s representing concrete functions to dispatch between.
///   This method will panic if `function_ids` is empty.
///
/// # Returns
/// The [FunctionId] of the new apply function
///
/// # Panics
/// If the `function_ids` argument has fewer than two elements, implying that no apply function is necessary.
fn create_apply_function(
    ssa: &mut Ssa,
    signature: Signature,
    caller_runtime: RuntimeType,
    function_ids: Vec<(FunctionId, RuntimeType)>,
) -> FunctionId {
    debug_assert!(
        function_ids.len() > 1,
        "create_apply_function is expected to be called with two or more FunctionIds"
    );
    // Clone the user-defined globals and the function purities mapping,
    // which are shared across all functions.
    // We will be borrowing `ssa` mutably so we need to fetch this shared information
    // before attempting to add a new function to the SSA.
    let globals = ssa.main().dfg.globals.clone();
    ssa.add_fn(|id| {
        let mut function_builder = FunctionBuilder::new("apply".to_string(), id);
        function_builder.set_globals(globals);

        // We want to push for apply functions to be inlined more aggressively;
        // they are expected to be optimized away by constants visible at the call site.
        let runtime = match caller_runtime {
            RuntimeType::Acir(_) => RuntimeType::Acir(InlineType::InlineAlways),
            RuntimeType::Brillig(_) => RuntimeType::Brillig(InlineType::InlineAlways),
        };
        function_builder.set_runtime(runtime);
        // Set up the parameters of the apply function
        // The first argument is the target function ID for which are dispatching a call
        let target_id = function_builder.add_parameter(Type::field());
        // The remaining apply function parameters are the actual parameters of the variants for which we are dispatching calls
        let params_ids = vecmap(signature.params, |typ| function_builder.add_parameter(typ));

        let entry_block = function_builder.current_block();

        let make_end_block = |builder: &mut FunctionBuilder| -> (BasicBlockId, Vec<ValueId>) {
            let block = builder.insert_block();
            // The values that will be ultimately returned from the function.
            let params =
                vecmap(&signature.returns, |typ| builder.add_block_parameter(block, typ.clone()));
            (block, params)
        };

        let (end_block, end_results) = make_end_block(&mut function_builder);
        let mut blocks_to_merge = Vec::with_capacity(function_ids.len());

        // Switch back to the entry block to build the rest of the dispatch function
        function_builder.switch_to_block(entry_block);

        for (index, (function_id, _)) in function_ids.iter().enumerate() {
            let is_last = index == function_ids.len() - 1;
            let mut next_function_block = None;

            let function_id_constant = function_builder
                .numeric_constant(function_id_to_field(*function_id), NumericType::NativeField);

            // If it's not the last function to dispatch, create an if statement
            if !is_last {
                next_function_block = Some(function_builder.insert_block());
                let executor_block = function_builder.insert_block();

                let condition =
                    function_builder.insert_binary(target_id, BinaryOp::Eq, function_id_constant);
                function_builder.terminate_with_jmpif(
                    condition,
                    executor_block,
                    next_function_block.unwrap(),
                );
                function_builder.switch_to_block(executor_block);
            } else {
                // Else just constrain the condition
                function_builder.insert_constrain(target_id, function_id_constant, None);
            }

            // Call the function variant
            let target_function_value = function_builder.import_function(*function_id);
            let call_results = function_builder
                .insert_call(target_function_value, params_ids.clone(), signature.returns.clone())
                .to_vec();

            let local_end_block = make_end_block(&mut function_builder);
            // Jump to the end block
            function_builder.terminate_with_jmp(local_end_block.0, call_results);
            blocks_to_merge.push(local_end_block);

            if let Some(next_block) = next_function_block {
                // Switch to the next block for the else branch
                function_builder.switch_to_block(next_block);
            }
        }

        // Merge blocks as last-in first-out:
        //
        // local_end_block0-----------------------------------------\
        //                                                           end block
        //                                                          /
        // local_end_block1---------------------\                  /
        //                                       new merge block2-/
        // local_end_block2--\                  /
        //                    new merge block1-/
        // local_end_block3--/
        //
        // This is necessary since SSA panics during flattening if we immediately
        // try to jump directly to end block instead
        // (see https://github.com/noir-lang/noir/issues/7323 for a case where this happens).
        //
        // It'd also be more efficient to merge them tournament-bracket style but that
        // also leads to panics during flattening for similar reasons.
        while let Some((block, results)) = blocks_to_merge.pop() {
            function_builder.switch_to_block(block);

            if let Some((block2, results2)) = blocks_to_merge.pop() {
                // Merge two blocks in the queue together
                let (new_merge, new_merge_results) = make_end_block(&mut function_builder);
                blocks_to_merge.push((new_merge, new_merge_results));

                function_builder.terminate_with_jmp(new_merge, results);

                function_builder.switch_to_block(block2);
                function_builder.terminate_with_jmp(new_merge, results2);
            } else {
                // Finally done, jump to the end
                function_builder.terminate_with_jmp(end_block, results);
            }
        }

        function_builder.switch_to_block(end_block);
        function_builder.terminate_with_return(end_results);

        // The above code can result in a suboptimal CFG so we simplify it here.
        let mut function = function_builder.current_function;
        function.simplify_function();
        function
    })
}

/// Creates a placeholder (dummy) function to replace calls to invalid function references.
/// An example of a possible invalid function reference is an out-of-bounds access on a zero-length function array.
///
/// This prevents the compiler from crashing by ensuring that the IR always has a valid function to call.
/// The dummy function is created using the supplied function's signature as to maintain a well formed SSA IR.
/// The dummy function is pure, contains no logic, and just returns zeroed out values for its return types.
///
/// This is especially useful in cases where we cannot statically resolve the function reference,
/// but want to continue compiling the rest of the program safely.
///
/// Returns the [FunctionId] of the newly created dummy function.
fn create_dummy_function(
    ssa: &mut Ssa,
    signature: Signature,
    caller_runtime: RuntimeType,
    purities: &mut HashMap<FunctionId, Purity>,
) -> FunctionId {
    ssa.add_fn(|id| {
        let mut function_builder = FunctionBuilder::new("apply_dummy".to_string(), id);

        // Set the runtime of the dummy function. The dummy function is expect to always be simplified out
        // but we let the caller set the runtime here as to match Noir's runtime semantics.
        let runtime = match caller_runtime {
            RuntimeType::Acir(_) => RuntimeType::Acir(InlineType::InlineAlways),
            RuntimeType::Brillig(_) => RuntimeType::Brillig(InlineType::InlineAlways),
        };
        function_builder.set_runtime(runtime);

        // The remaining dummy function parameters are the actual parameters of the function call without any variants.
        // We generate these just to maintain a well-formed IR. Not doing this could result in errors if the dummy function
        // was set to be inlined before the call to it was removed by DIE.
        vecmap(signature.params, |typ| function_builder.add_parameter(typ));

        // We can mark the dummy function pure as all it does is return.
        // As the dummy function is just meant to be a placeholder for any calls to
        // higher-order functions without variants, we want the function to be marked pure
        // so that dead instruction elimination can remove any calls to it.
        purities.insert(id, Purity::Pure);

        let results =
            vecmap(signature.returns, |typ| make_dummy_return_data(&mut function_builder, &typ));

        function_builder.terminate_with_return(results);

        function_builder.current_function
    })
}

/// Construct a dummy value to be returned from the placeholder function for calls to invalid lambda references.
/// We need to construct the appropriate value so that our SSA is well formed even if the function
/// pointer has no variants.
fn make_dummy_return_data(function_builder: &mut FunctionBuilder, typ: &Type) -> ValueId {
    match typ {
        Type::Numeric(numeric_type) => function_builder.numeric_constant(0_u128, *numeric_type),
        Type::Array(element_types, len) => {
            let mut array = im::Vector::new();
            for _ in 0..*len {
                for typ in element_types.iter() {
                    array.push_back(make_dummy_return_data(function_builder, typ));
                }
            }
            function_builder.insert_make_array(array, typ.clone())
        }
        Type::Vector(_) => {
            let array = im::Vector::new();
            // The contents of a vector do not matter for a dummy function, we simply
            // desire to have a well formed SSA by returning the correct value for a type.
            // Thus, we return an empty vector here.
            function_builder.insert_make_array(array, typ.clone())
        }
        Type::Reference(element_type) => function_builder.insert_allocate((**element_type).clone()),
        Type::Function => {
            unreachable!(
                "ICE: Any function passed as a value should have already been converted to a field type"
            )
        }
    }
}

/// Check pre-execution properties.
///
/// Panics if:
///   * Any intrinsic or foreign function is passed as a value.
#[cfg(debug_assertions)]
fn defunctionalize_pre_check(function: &Function) {
    visit_values_other_than_call_target(function, |value| match value {
        Value::ForeignFunction(name) => panic!("foreign function as value: {name}"),
        Value::Intrinsic(intrinsic) => panic!("intrinsic function as value: {intrinsic}"),
        _ => (),
    });
}

/// Check post-execution properties:
/// * All blocks which took function parameters should receive a discriminator instead
#[cfg(debug_assertions)]
fn defunctionalize_post_check(func: &Function) {
    for block_id in func.reachable_blocks() {
        for param in func.dfg[block_id].parameters() {
            let value = &func.dfg[*param];
            let Value::Param { typ, .. } = value else {
                panic!("unexpected parameter value: {value:?}");
            };
            assert!(
                replacement_type(typ).is_none(),
                "Blocks are not expected to take function parameters any more. Got '{typ}' in param {param} of block {block_id} in function {} {}",
                func.name(),
                func.id()
            );
        }
    }
}

/// Return what type a function value type should be replaced with:
/// * Global functions are replaced with a `Field`.
/// * Function references are replaced with a reference to the replacement type of the underlying type, recursively.
/// * Array and vectors that contain function types are handled recursively.
///
/// If the type doesn't need replacement, `None` is returned.
fn replacement_type(typ: &Type) -> Option<Type> {
    match typ {
        Type::Function => Some(Type::field()),
        Type::Reference(typ) => {
            replacement_type(typ.as_ref()).map(|typ| Type::Reference(Arc::new(typ)))
        }
        Type::Numeric(_) => None,
        Type::Array(items, size) => {
            replacement_types(items.as_ref()).map(|types| Type::Array(Arc::new(types), *size))
        }
        Type::Vector(items) => {
            replacement_types(items.as_ref()).map(|types| Type::Vector(Arc::new(types)))
        }
    }
}

/// Take a vector of types that might need replacement.
/// Replaces the ones that need replacement, leaving all others as-is.
/// If no type needs replacement, `None` is returned.
fn replacement_types(types: &[Type]) -> Option<Vec<Type>> {
    let mut reps = Vec::new();
    let mut has_rep = false;
    for typ in types {
        let rep = replacement_type(typ);
        has_rep |= rep.is_some();
        reps.push(rep);
    }
    if !has_rep {
        None
    } else {
        Some(
            reps.into_iter()
                .zip(types)
                .map(|(rep, typ)| rep.unwrap_or_else(|| typ.clone()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{
                IResults,
                tests::expect_value_with_args,
                value::{NumericValue, Value},
            },
            ir::function::FunctionId,
            opt::defunctionalize::create_apply_functions,
        },
    };

    use super::{Ssa, find_variants};

    #[test]
    fn apply_inherits_caller_runtime() {
        // Extracted from `execution_success/brillig_fns_as_values` with `--force-brillig`
        // with an additional simple higher-order function
        let src = "
          brillig(inline) fn main f0 {
            b0(v0: u32):
              v3 = call f1(f2, v0) -> u32
              v5 = add v0, u32 1
              v6 = eq v3, v5
              constrain v3 == v5
              v8 = call f1(f3, v0) -> u32
              v9 = add v0, u32 1
              v10 = eq v8, v9
              constrain v8 == v9
              v11 = call f1(f4, v0) -> u32
              v12 = add v0, u32 1
              constrain v11 == v12
              return
          }
          brillig(inline) fn wrapper f1 {
            b0(v0: function, v1: u32):
              v2 = call v0(v1) -> u32
              return v2
          }
          brillig(inline) fn increment f2 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
          brillig(inline) fn increment_acir f3 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
          brillig(inline) fn increment_three f4 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v3 = call f1(Field 2, v0) -> u32
            v5 = add v0, u32 1
            v6 = eq v3, v5
            constrain v3 == v5
            v8 = call f1(Field 3, v0) -> u32
            v9 = add v0, u32 1
            v10 = eq v8, v9
            constrain v8 == v9
            v12 = call f1(Field 4, v0) -> u32
            v13 = add v0, u32 1
            constrain v12 == v13
            return
        }
        brillig(inline) fn wrapper f1 {
          b0(v0: Field, v1: u32):
            v3 = call f5(v0, v1) -> u32
            return v3
        }
        brillig(inline) fn increment f2 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        brillig(inline) fn increment_acir f3 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        brillig(inline) fn increment_three f4 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        brillig(inline_always) fn apply f5 {
          b0(v0: Field, v1: u32):
            v5 = eq v0, Field 2
            jmpif v5 then: b2, else: b1
          b1():
            v9 = eq v0, Field 3
            jmpif v9 then: b4, else: b3
          b2():
            v7 = call f2(v1) -> u32
            jmp b6(v7)
          b3():
            constrain v0 == Field 4
            v14 = call f4(v1) -> u32
            jmp b5(v14)
          b4():
            v11 = call f3(v1) -> u32
            jmp b5(v11)
          b5(v2: u32):
            jmp b6(v2)
          b6(v3: u32):
            return v3
        }
        ");
    }

    #[test]
    fn apply_created_per_caller_runtime() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3 = call f1(f3, v0) -> u32
            v5 = call f1(f6, v0) -> u32
            v7 = add v0, u32 1
            v8 = eq v3, v7
            constrain v3 == v7
            v11 = call f2(f4, v0) -> u32
            v13 = call f2(f5, v0) -> u32
            v14 = add v0, u32 1
            v15 = eq v11, v14
            constrain v11 == v14
            return
        }
        brillig(inline) fn wrapper f1 {
          b0(v0: function, v1: u32):
            v2 = call v0(v1) -> u32
            return v2
        }
        acir(inline) fn wrapper_acir f2 {
          b0(v0: function, v1: u32):
            v2 = call v0(v1) -> u32
            return v2
        }
        brillig(inline) fn increment f3 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        acir(inline) fn increment_acir f4 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        acir(inline) fn increment_acir1 f5 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        brillig(inline) fn increment_brillig1 f6 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        let applies = ssa.functions.values().filter(|f| f.name() == "apply").collect::<Vec<_>>();
        assert!(applies.iter().any(|f| f.runtime().is_acir()));
        assert!(applies.iter().any(|f| f.runtime().is_brillig()));
    }

    #[test]
    fn apply_created_for_stored_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut function
            store f1 at v1
            jmpif v0 then: b1, else: b2
          b1():
            store f2 at v1
            jmp b2()
          b2():
            v4 = load v1 -> function
            v6 = call f3(v4) -> u32
            return v6
        }
        acir(inline) fn foo f1 {
          b0():
            return u32 1
        }
        acir(inline) fn bar f2 {
          b0():
            return u32 2
        }
        acir(inline) fn caller f3 {
          b0(v0: function):
            v1 = call v0() -> u32
            v2 = call v0() -> u32
            v3 = add v1, v2
            return v3
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(
            ssa,
            @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 1 at v1
            jmpif v0 then: b1, else: b2
          b1():
            store Field 2 at v1
            jmp b2()
          b2():
            v4 = load v1 -> Field
            v6 = call f3(v4) -> u32
            return v6
        }
        acir(inline) fn foo f1 {
          b0():
            return u32 1
        }
        acir(inline) fn bar f2 {
          b0():
            return u32 2
        }
        acir(inline) fn caller f3 {
          b0(v0: Field):
            v2 = call f4(v0) -> u32
            v3 = call f4(v0) -> u32
            v4 = add v2, v3
            return v4
        }
        acir(inline_always) fn apply f4 {
          b0(v0: Field):
            v3 = eq v0, Field 1
            jmpif v3 then: b2, else: b1
          b1():
            constrain v0 == Field 2
            v8 = call f2() -> u32
            jmp b3(v8)
          b2():
            v5 = call f1() -> u32
            jmp b3(v5)
          b3(v1: u32):
            return v1
        }
        "
        );
    }

    // This test shows that the following SSA returns a `NumericValue::Field` before
    // `defunctionalize` is run, but returns a `FunctionId` after defunctionalize,
    // when interpreted.
    //
    // Note that the ACIR 'main' fn returns a function
    // (expected to be disallowed by the frontend).
    #[test]
    fn interpret_acir_returning_fn() {
        let src = "
          acir(inline) fn main f0 {
            b0():
              call f1()
              return f1
          }
          acir(inline) fn bar f1 {
            b0():
              return
          }
        ";

        let defunctionalize_ssa = Ssa::from_str(src).unwrap();
        let defunctionalize_ssa = defunctionalize_ssa.defunctionalize();
        let defunctionalize_results = defunctionalize_ssa.interpret(vec![]);

        let interpreter_return_values = expect_value_with_args(src, vec![]);

        let expected_interpreter_return_values = Value::Function(FunctionId::test_new(1));
        let expected_defunctionalize_results: IResults =
            Ok(vec![Value::Numeric(NumericValue::Field(1u128.into()))]);

        assert_eq!(defunctionalize_results, expected_defunctionalize_results);
        assert_eq!(interpreter_return_values, expected_interpreter_return_values);

        assert_ssa_snapshot!(defunctionalize_ssa, @r"
          acir(inline) fn main f0 {
            b0():
              call f1()
              return Field 1
          }
          acir(inline) fn bar f1 {
            b0():
              return
          }
        ");
    }

    // Test from SSA fuzzing to check behavior of 'defunctionalize' pass on
    // an ACIR 'main' fn that accepts and returns a function parameter
    // (expected to be disallowed by the frontend).
    //
    // When fuzzing, it was panicking with:
    // "Could not find apply function"
    #[test]
    fn missing_fn_parameter_type() {
        let src = "
          acir(inline) fn get_t_c f3 {
            b0(v0: function):
              v1 = call v0() -> function
              return v1
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn get_t_c f0 {
          b0(v0: Field):
            v2 = call f1() -> Field
            return v2
        }
        acir(inline_always) pure fn apply_dummy f1 {
          b0():
            return Field 0
        }
        ");
    }

    // Test from SSA fuzzing to check behavior of 'defunctionalize' pass on
    // a Brillig 'main' fn that accepts a function parameter (expected to be
    // disallowed by the frontend).
    #[test]
    fn missing_fn() {
        let src = "
          brillig(inline) fn main f0 {
            b0(v0: function, v1: u32):
              v2 = call v0(v1) -> u32
              return v2
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v3 = call f1(v1) -> u32
            return v3
        }
        brillig(inline_always) pure fn apply_dummy f1 {
          b0(v0: u32):
            return u32 0
        }
        ");
    }

    #[test]
    fn missing_fn_variant() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = call f1(f2) -> i64
            return v2
        }
        brillig(inline) fn func_3 f1 {
          b0(v0: function):
            return i64 0
        }
        brillig(inline) fn func_2 f2 {
          b0(v0: function):
            v2 = call v0(u128 1) -> u1
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        // We still expect all parameters with a function type to be replaced.
        // However, this is fine as a function with no variants means that function
        // was never actually called.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v2 = call f1(Field 2) -> i64
            return v2
        }
        brillig(inline) fn func_3 f1 {
          b0(v0: Field):
            return i64 0
        }
        brillig(inline) fn func_2 f2 {
          b0(v0: Field):
            v3 = call f3(u128 1) -> u1
            return v3
        }
        brillig(inline_always) pure fn apply_dummy f3 {
          b0(v0: u128):
            return u1 0
        }
        ");
    }

    #[test]
    fn apply_function_with_dynamic_dispatch_id() {
        // This checks that we generate an apply function correctly when
        // we have a dynamic dispatch.
        // The flattening pass
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = sub v0, u32 1
            v4 = call f2(v2) -> u32
            return v4
        }
        acir(inline) fn lambdas_with_input_and_return_values f1 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(f2)
          b2():
            v6 = eq v0, u32 1
            jmpif v6 then: b4, else: b5
          b3(v1: function):
            v10 = call v1(v0) -> u32
            return v10
          b4():
            jmp b6(f3)
          b5():
            jmp b6(f4)
          b6(v2: function):
            jmp b3(v2)
        }
        acir(inline) fn lambda f2 {
          b0(v0: u32):
            return v0
        }
        acir(inline) fn lambda f3 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        acir(inline) fn lambda f4 {
          b0(v0: u32):
            v2 = add v0, u32 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = sub v0, u32 1
            v4 = call f2(v2) -> u32
            return v4
        }
        acir(inline) fn lambdas_with_input_and_return_values f1 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(Field 2)
          b2():
            v6 = eq v0, u32 1
            jmpif v6 then: b4, else: b5
          b3(v1: Field):
            v11 = call f5(v1, v0) -> u32
            return v11
          b4():
            jmp b6(Field 3)
          b5():
            jmp b6(Field 4)
          b6(v2: Field):
            jmp b3(v2)
        }
        acir(inline) fn lambda f2 {
          b0(v0: u32):
            return v0
        }
        acir(inline) fn lambda f3 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        acir(inline) fn lambda f4 {
          b0(v0: u32):
            v2 = add v0, u32 2
            return v2
        }
        acir(inline_always) fn apply f5 {
          b0(v0: Field, v1: u32):
            v5 = eq v0, Field 2
            jmpif v5 then: b2, else: b1
          b1():
            v9 = eq v0, Field 3
            jmpif v9 then: b4, else: b3
          b2():
            v7 = call f2(v1) -> u32
            jmp b6(v7)
          b3():
            constrain v0 == Field 4
            v14 = call f4(v1) -> u32
            jmp b5(v14)
          b4():
            v11 = call f3(v1) -> u32
            jmp b5(v11)
          b5(v2: u32):
            jmp b6(v2)
          b6(v3: u32):
            return v3
        }
        ");
    }

    #[test]
    fn fn_in_array() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v5 = make_array [f1, f2, f3, f4] : [function; 4]
            v7 = lt v0, u32 4
            constrain v7 == u1 1, "Index out of bounds"
            v9 = array_get v5, index v0 -> function
            call v9()
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }
        acir(inline) fn lambda f3 {
          b0():
            return
        }
        acir(inline) fn lambda f4 {
          b0():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v5 = make_array [Field 1, Field 2, Field 3, Field 4] : [Field; 4]
            v7 = lt v0, u32 4
            constrain v7 == u1 1, "Index out of bounds"
            v9 = array_get v5, index v0 -> Field
            call f5(v9)
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }
        acir(inline) fn lambda f3 {
          b0():
            return
        }
        acir(inline) fn lambda f4 {
          b0():
            return
        }
        acir(inline_always) fn apply f5 {
          b0(v0: Field):
            v2 = eq v0, Field 1
            jmpif v2 then: b2, else: b1
          b1():
            v5 = eq v0, Field 2
            jmpif v5 then: b4, else: b3
          b2():
            call f1()
            jmp b9()
          b3():
            v8 = eq v0, Field 3
            jmpif v8 then: b6, else: b5
          b4():
            call f2()
            jmp b8()
          b5():
            constrain v0 == Field 4
            call f4()
            jmp b7()
          b6():
            call f3()
            jmp b7()
          b7():
            jmp b8()
          b8():
            jmp b9()
          b9():
            return
        }
        "#);
    }

    #[test]
    fn empty_make_array_with_functions() {
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [function; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> function
            v6, v7 = call v4(u32 5) -> (u32, [u32; 2])
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        // Guarantee we make the following updates:
        // 1. The make_array instruction type is modified
        // 2. We generate a dummy function which is used to modify function calls when there are no variants
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [Field; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> Field
            v7, v8 = call f1(u32 5) -> (u32, [u32; 2])
            return
        }
        acir(inline_always) pure fn apply_dummy f1 {
          b0(v0: u32):
            v2 = make_array [u32 0, u32 0] : [u32; 2]
            return u32 0, v2
        }
        "#);
    }

    #[test]
    fn empty_make_array_with_functions_returning_functions() {
        let src = "
      acir(inline) fn main f0 {
        b0():
          v0 = make_array [] : [function; 0]
          constrain u1 0 == u1 1
          v4 = array_get v0, index u32 0 -> function
          v6 = call v4(u32 2) -> function
          return
      }
      ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [Field; 0]
            constrain u1 0 == u1 1
            v4 = array_get v0, index u32 0 -> Field
            v7 = call f1(u32 2) -> Field
            return
        }
        acir(inline_always) pure fn apply_dummy f1 {
          b0(v0: u32):
            return Field 0
        }
        ");
    }

    #[test]
    fn mut_ref_function() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut function
            store f1 at v0
            v3 = call f2(v0) -> u1
            return v3
        }
        acir(inline) fn bar f1 {
          b0():
            return u1 0
        }
        acir(inline) fn foo f2 {
          b0(v0: &mut function):
            v1 = load v0 -> function
            v2 = call v1() -> u1
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v3 = call f2(v0) -> u1
            return v3
        }
        acir(inline) fn bar f1 {
          b0():
            return u1 0
        }
        acir(inline) fn foo f2 {
          b0(v0: &mut Field):
            v1 = load v0 -> Field
            v3 = call f1() -> u1
            return v3
        }
        ");
    }

    /// Test case showing a function that takes two lambdas:
    /// one by value, one by mutable reference.
    #[test]
    fn mixed_signature() {
        let src = "
        brillig(inline) fn main f0 {
        b0():
            v0 = allocate -> &mut function
            store f1 at v0
            v4 = call f3(v0, f2) -> &mut function
            return
        }
        brillig(inline) fn lambda f1 {
        b0():
            return Field 0
        }
        brillig(inline) fn lambda f2 {
        b0(v0: &mut function):
            return v0
        }
        brillig(inline) fn get f3 {
        b0(v0: &mut function, v1: function):
            v2 = call v1(v0) -> &mut function
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v4 = call f3(v0, Field 2) -> &mut Field
            return
        }
        brillig(inline) fn lambda f1 {
          b0():
            return Field 0
        }
        brillig(inline) fn lambda f2 {
          b0(v0: &mut Field):
            return v0
        }
        brillig(inline) fn get f3 {
          b0(v0: &mut Field, v1: Field):
            v3 = call f2(v0) -> &mut Field
            return v3
        }
        ");
    }

    #[test]
    fn find_functions_as_values_finds_function_in_array_set() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = make_array [f1] : [function; 1]
            v2 = array_set v1, index u32 0, value f2
            return v0
        }

        acir(inline) fn foo f1 {
          b0(v0: u32):
            return v0
        }

        acir(inline) fn bar f2 {
          b0(v0: u32):
            return v0
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let functions = super::find_functions_as_values(main);
        assert_eq!(functions.len(), 2);
        assert!(functions.contains(&FunctionId::test_new(1))); // foo
        assert!(functions.contains(&FunctionId::test_new(2))); // bar
    }

    /// Ensure apply function are cached with the signature of the function before any mutations occur
    #[test]
    fn regression_8896() {
        let src = r#"
            acir(inline) fn main f0 {
            b0(v0: Field):
                v3 = call f1(f2) -> Field
                v5 = call f1(f3) -> Field
                v7 = eq v0, Field 0
                jmpif v7 then: b1, else: b2
            b1():
                jmp b3(f4)
            b2():
                jmp b3(f5)
            b3(v10: function):
                v11 = add v3, v5
                v12 = call v10(v0) -> Field
                v13 = add v11, v12
                return v13
            }
            acir(inline) fn dispatch1 f1 {
            b0(v0: function):
                v2 = call v0(f6) -> Field
                v4 = mul v2, Field 3
                return v4
            }
            acir(inline) fn lambda f2 {
            b0(v0: function):
                return Field 1
            }
            acir(inline) fn lambda f3 {
            b0(v0: function):
                return Field 2
            }
            acir(inline) fn fn1 f4 {
            b0(v0: Field):
                v2 = add v0, Field 1
                return v2
            }
            acir(inline) fn fn2 f5 {
            b0(v0: Field):
                v2 = mul v0, Field 5
                return v2
            }
            acir(inline) fn lambda f6 {
            b0():
                return Field 0
            }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v4 = call f1(Field 2) -> Field
            v6 = call f1(Field 3) -> Field
            v8 = eq v0, Field 0
            jmpif v8 then: b1, else: b2
          b1():
            jmp b3(Field 4)
          b2():
            jmp b3(Field 5)
          b3(v1: Field):
            v11 = add v4, v6
            v13 = call f7(v1, v0) -> Field
            v14 = add v11, v13
            return v14
        }
        acir(inline) fn dispatch1 f1 {
          b0(v0: Field):
            v3 = call f8(v0, Field 6) -> Field
            v5 = mul v3, Field 3
            return v5
        }
        acir(inline) fn lambda f2 {
          b0(v0: Field):
            return Field 1
        }
        acir(inline) fn lambda f3 {
          b0(v0: Field):
            return Field 2
        }
        acir(inline) fn fn1 f4 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        acir(inline) fn fn2 f5 {
          b0(v0: Field):
            v2 = mul v0, Field 5
            return v2
        }
        acir(inline) fn lambda f6 {
          b0():
            return Field 0
        }
        acir(inline_always) fn apply f7 {
          b0(v0: Field, v1: Field):
            v4 = eq v0, Field 4
            jmpif v4 then: b2, else: b1
          b1():
            constrain v0 == Field 5
            v9 = call f5(v1) -> Field
            jmp b3(v9)
          b2():
            v6 = call f4(v1) -> Field
            jmp b3(v6)
          b3(v2: Field):
            return v2
        }
        acir(inline_always) fn apply f8 {
          b0(v0: Field, v1: Field):
            v4 = eq v0, Field 2
            jmpif v4 then: b2, else: b1
          b1():
            constrain v0 == Field 3
            v9 = call f3(v1) -> Field
            jmp b3(v9)
          b2():
            v6 = call f2(v1) -> Field
            jmp b3(v6)
          b3(v2: Field):
            return v2
        }
        ");
    }

    /// Ensure the correct type signature is used for recursive calls. We should expect 2 apply
    /// functions generated, not one.
    #[test]
    fn regression_8897() {
        let src = r#"
            acir(inline) fn main f0 {
            b0():
                v3 = call f1(f2, Field 0) -> Field
                return v3
            }
            acir(inline) fn simple_recur f1 {
            b0(v0: function, v1: Field):
                v3 = eq v1, Field 0
                jmpif v3 then: b1, else: b2
            b1():
                jmp b3(f1)
            b2():
                jmp b3(f3)
            b3(v6: function):
                v9 = add v1, Field 1
                v10 = call v6(f4, v9) -> Field
                v11 = call v0(v10, Field 0) -> Field
                return v11
            }
            acir(inline) fn fn1 f2 {
            b0(v0: Field, v1: Field):
                v3 = add v0, Field 1
                return v3
            }
            acir(inline) fn lambda f3 {
            b0(v0: function, v1: Field):
                v4 = call f2(Field 0, Field 0) -> Field
                return v4
            }
            acir(inline) fn fn2 f4 {
            b0(v0: Field, v1: Field):
                v3 = mul v1, Field 5
                return v3
            }
        "#;

        let mut ssa = Ssa::from_str(src).unwrap();
        let variants = find_variants(&ssa);
        assert_eq!(variants.len(), 2);

        let (apply_functions, _purities) = create_apply_functions(&mut ssa, variants);
        // This was 1 before this bug was fixed.
        assert_eq!(apply_functions.len(), 2);
    }

    #[test]
    fn acir_variant_in_brillig_last_function_to_dispatch() {
        //The src code below correspond to this code:
        // let f = if x == 1 { foo1 } else { foo };
        // fn foo(x: Field) -> Field {
        //     x * x
        // }
        // unconstrained fn foo1(x: Field) -> Field {
        //     x * x
        // }
        // This code is invalid because foo1 and foo do not have compatible signatures.
        // However, some generated code (e.g the zeroed impls generated by the frontend) may produce such code.
        // We ensure that the apply function does not allow calling acir functions from a brillig function
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v4 = allocate -> &mut Field
            store v0 at v4
            v5 = load v4 -> Field
            v7 = eq v5, Field 1
            jmpif v7 then: b1, else: b2
          b1():
            jmp b3(f1)
          b2():
            jmp b3(f3)
          b3(v3: function):
            v10 = load v4 -> Field
            v12 = call f3(v10) -> Field
            v13 = call v3(v12) -> Field
            return
        }
        brillig(inline) fn foo1 f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        brillig(inline) fn foo2 f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        acir(inline) fn foo f3 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
       "#;

        let ssa = Ssa::from_str_no_validation(src).unwrap();

        let mut variants = find_variants(&ssa);
        let ((_, caller_runtime), variants) = variants.pop_last().unwrap();
        assert!(caller_runtime.is_brillig());
        assert!(variants[0].1.is_brillig());
        assert!(variants[1].1.is_acir());

        let ssa = ssa.defunctionalize();

        // The `apply` method skips calling the acir function f3.
        // As there is only one other variant, we just call f1 directly.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v4 = allocate -> &mut Field
            store v0 at v4
            v5 = load v4 -> Field
            v7 = eq v5, Field 1
            jmpif v7 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 3)
          b3(v3: Field):
            v9 = load v4 -> Field
            v11 = call f3(v9) -> Field
            v13 = call f1(v11) -> Field
            return
        }
        brillig(inline) fn foo1 f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        brillig(inline) fn foo2 f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        acir(inline) fn foo f3 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        ");
    }

    /// This test expands [acir_variant_in_brillig_last_function_to_dispatch] by having multiple
    /// ACIR variants be at the end of the proposed variant dispatch table
    #[test]
    fn acir_variant_in_brillig_multiple_at_end_are_skipped() {
        // Pseudocode of the intended semantics:
        // let f = if x == 1 { foo1 } else if x == 2 { foo2 } else if x == 3 { foo3 } else { foo4 };
        //
        // - foo1, foo2 are Brillig functions
        // - foo3, foo4 are ACIR functions
        //
        // At runtime, main may pass either f3 or f4 as the closure target `v2`.
        // The apply function will skip those calls, but still constrain the target ID
        // so that if f3/f4 are ever used, the program fails.
        // If f3/f4 are ever used it means there is a bug upstream in the frontend.
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v4 = eq v0, Field 1
            jmpif v4 then: b1, else: b2
          b1():
            jmp b7(f1)
          b2():
            v6 = eq v0, Field 2
            jmpif v6 then: b3, else: b4
          b3():
            jmp b7(f2)
          b4():
            v8 = eq v0, Field 3
            jmpif v8 then: b5, else: b6
          b5():
            jmp b7(f3)
          b6():
            jmp b7(f4)
          b7(v2: function):
            v13 = call f1(v0) -> Field
            v14 = call v2(v13) -> Field
            return
        }
        brillig(inline) fn foo1 f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        brillig(inline) fn foo2 f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        acir(inline) fn foo3 f3 {
          b0(v0: Field):
            v1 = sub v0, v0
            return v1
        }
        acir(inline) fn foo4 f4 {
          b0(v0: Field):
            v1 = div v0, v0
            return v1
        }
        "#;

        let ssa = Ssa::from_str_no_validation(src).unwrap();

        let mut variants = find_variants(&ssa);
        let ((_, caller_runtime), variants) = variants.pop_last().unwrap();
        assert!(caller_runtime.is_brillig());
        assert!(variants[0].1.is_brillig());
        assert!(variants[1].1.is_brillig());
        assert!(variants[2].1.is_acir());
        assert!(variants[3].1.is_acir());

        let ssa = ssa.defunctionalize();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v4 = eq v0, Field 1
            jmpif v4 then: b1, else: b2
          b1():
            jmp b7(Field 1)
          b2():
            v6 = eq v0, Field 2
            jmpif v6 then: b3, else: b4
          b3():
            jmp b7(Field 2)
          b4():
            v8 = eq v0, Field 3
            jmpif v8 then: b5, else: b6
          b5():
            jmp b7(Field 3)
          b6():
            jmp b7(Field 4)
          b7(v2: Field):
            v11 = call f1(v0) -> Field
            v13 = call f5(v2, v11) -> Field
            return
        }
        brillig(inline) fn foo1 f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        brillig(inline) fn foo2 f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        acir(inline) fn foo3 f3 {
          b0(v0: Field):
            v1 = sub v0, v0
            return v1
        }
        acir(inline) fn foo4 f4 {
          b0(v0: Field):
            v1 = div v0, v0
            return v1
        }
        brillig(inline_always) fn apply f5 {
          b0(v0: Field, v1: Field):
            v4 = eq v0, Field 1
            jmpif v4 then: b2, else: b1
          b1():
            constrain v0 == Field 2
            v9 = call f2(v1) -> Field
            jmp b3(v9)
          b2():
            v6 = call f1(v1) -> Field
            jmp b3(v6)
          b3(v2: Field):
            return v2
        }
        ");
    }

    #[test]
    fn acir_variant_in_brillig_not_last_to_dispatch() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(f1, f2)
          b2():
            v6, v7 = call f3() -> (function, function)
            jmp b3(v6, v7)
          b3(v1: function, v2: function):
            call v2()
            return
        }
        brillig(inline) fn lambda f1 {
          b0():
            return
        }
        brillig(inline) fn lambda f2 {
          b0():
            return
        }
        brillig(inline) fn return_lambda f3 {
          b0():
            return f4, f5
        }
        acir(inline) fn zeroed_lambda f4 {
          b0():
            return
        }
        brillig(inline) fn zeroed_lambda f5 {
          b0():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        // If Field 4 were ever to be passed to `apply` the program would fail.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(Field 1, Field 2)
          b2():
            v6, v7 = call f3() -> (Field, Field)
            jmp b3(v6, v7)
          b3(v1: Field, v2: Field):
            call f6(v2)
            return
        }
        brillig(inline) fn lambda f1 {
          b0():
            return
        }
        brillig(inline) fn lambda f2 {
          b0():
            return
        }
        brillig(inline) fn return_lambda f3 {
          b0():
            return Field 4, Field 5
        }
        acir(inline) fn zeroed_lambda f4 {
          b0():
            return
        }
        brillig(inline) fn zeroed_lambda f5 {
          b0():
            return
        }
        brillig(inline_always) fn apply f6 {
          b0(v0: Field):
            v2 = eq v0, Field 1
            jmpif v2 then: b2, else: b1
          b1():
            v5 = eq v0, Field 2
            jmpif v5 then: b4, else: b3
          b2():
            call f1()
            jmp b5()
          b3():
            constrain v0 == Field 5
            call f5()
            jmp b5()
          b4():
            call f2()
            jmp b5()
          b5():
            return
        }
        ");
    }

    #[test]
    fn brillig_variant_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(f1, f2)
          b2():
            v6, v7 = call f3() -> (function, function)
            jmp b3(v6, v7)
          b3(v1: function, v2: function):
            call v2()
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }
        acir(inline) fn return_lambda f3 {
          b0():
            return f4, f5
        }
        brillig(inline) fn zeroed_lambda f4 {
          b0():
            return
        }
        acir(inline) fn zeroed_lambda f5 {
          b0():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        // A call to f4 will be created in the `apply` function in this case.
        // It is valid to call Brillig from ACIR as this will be treated as a new Brillig entry point.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(Field 1, Field 2)
          b2():
            v6, v7 = call f3() -> (Field, Field)
            jmp b3(v6, v7)
          b3(v1: Field, v2: Field):
            call f6(v2)
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }
        acir(inline) fn return_lambda f3 {
          b0():
            return Field 4, Field 5
        }
        brillig(inline) fn zeroed_lambda f4 {
          b0():
            return
        }
        acir(inline) fn zeroed_lambda f5 {
          b0():
            return
        }
        acir(inline_always) fn apply f6 {
          b0(v0: Field):
            v2 = eq v0, Field 1
            jmpif v2 then: b2, else: b1
          b1():
            v5 = eq v0, Field 2
            jmpif v5 then: b4, else: b3
          b2():
            call f1()
            jmp b9()
          b3():
            v8 = eq v0, Field 4
            jmpif v8 then: b6, else: b5
          b4():
            call f2()
            jmp b8()
          b5():
            constrain v0 == Field 5
            call f5()
            jmp b7()
          b6():
            call f4()
            jmp b7()
          b7():
            jmp b8()
          b8():
            jmp b9()
          b9():
            return
        }
        ");
    }
}
