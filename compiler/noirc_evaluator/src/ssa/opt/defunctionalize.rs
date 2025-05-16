//! This module defines the defunctionalization pass for the SSA IR.
//! Certain IR targets (e.g., Brillig and ACIR) do not support higher-order functions directly.
//!
//! The pass eliminates higher-order functions (a function which accepts function values as arguments or returns functions)
//! by transforming functions used as values (i.e., first-class functions)
//! into constant numbers (fields) that represent their function IDs.
//!
//! Defunctionalization handles higher-order functions functions by lowering function values into
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
//!         1 -> function0(arg1, arg2),
//!         ...
//!         N -> functionN(arg1, arg2),
//!     }
//! }
//! ```
//!
//! After this pass all first-class functions are replaced with numeric IDs
//! and calls are routed via the newly generated `apply` functions.
use std::collections::{BTreeMap, BTreeSet};

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
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

/// Represents an 'apply' function created by this pass to dispatch higher order functions to.
/// Pseudocode of an `apply` function is given below:
/// ```text
/// fn apply(function_id: Field, arg1: Field, arg2: Field) -> Field {
///     match function_id {
///         0 -> function0(arg1, arg2),
///         1 -> function0(arg1, arg2),
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
/// Maps ([Signature], [RuntimeType]) -> Vec<[FunctionId]>
type Variants = BTreeMap<(Signature, RuntimeType), Vec<FunctionId>>;
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
    // TODO revert?
    // pub(crate) fn defunctionalize(mut self) -> Ssa {
    pub fn defunctionalize(mut self) -> Ssa {
        // Find all functions used as value that share the same signature and runtime type
        let variants = find_variants(&self);

        // Generate the apply functions for the provided variants
        let apply_functions = create_apply_functions(&mut self, variants);

        // Setup the pass context
        let context = DefunctionalizationContext { apply_functions };

        // Run defunctionalization over all functions in the SSA
        context.defunctionalize_all(&mut self);
        self
    }
}

impl DefunctionalizationContext {
    /// Defunctionalize all functions in the Ssa
    fn defunctionalize_all(mut self, ssa: &mut Ssa) {
        for function in ssa.functions.values_mut() {
            self.defunctionalize(function);
        }
    }

    /// Defunctionalize a single function
    fn defunctionalize(&mut self, func: &mut Function) {
        for block_id in func.reachable_blocks() {
            let block = &mut func.dfg[block_id];

            // Temporarily take the parameters here just to avoid cloning them
            let parameters = block.take_parameters();
            for parameter in &parameters {
                if func.dfg.type_of_value(*parameter) == Type::Function {
                    func.dfg.set_type_of_value(*parameter, Type::field());
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
            // each first class function with a field value and replacing calls
            // to a first class function to a call to the relevant `apply` function.
            #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
            for instruction_id in block.instructions().to_vec() {
                let mut instruction = func.dfg[instruction_id].clone();
                let mut replacement_instruction = None;

                if remove_first_class_functions_in_instruction(func, &mut instruction) {
                    func.dfg[instruction_id] = instruction.clone();
                }

                #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
                for result in func.dfg.instruction_results(instruction_id).to_vec() {
                    if func.dfg.type_of_value(result) == Type::Function {
                        func.dfg.set_type_of_value(result, Type::field());
                    }
                }

                // Operate on call instructions
                let (target_func_id, arguments) = match &instruction {
                    Instruction::Call { func: target_func_id, arguments } => {
                        (*target_func_id, arguments)
                    }
                    _ => {
                        continue;
                    }
                };

                match func.dfg[target_func_id] {
                    // If the target is a function used as value
                    Value::Param { .. } | Value::Instruction { .. } => {
                        let mut arguments = arguments.clone();
                        let results = func.dfg.instruction_results(instruction_id);
                        let signature = Signature {
                            params: vecmap(&arguments, |param| func.dfg.type_of_value(*param)),
                            returns: vecmap(results, |result| func.dfg.type_of_value(*result)),
                        };

                        // Find the correct apply function
                        let apply_function = self.get_apply_function(signature, func.runtime());

                        // Replace the instruction with a call to apply
                        let apply_function_value_id = func.dfg.import_function(apply_function.id);
                        if apply_function.dispatches_to_multiple_functions {
                            arguments.insert(0, target_func_id);
                        }
                        let func = apply_function_value_id;
                        replacement_instruction = Some(Instruction::Call { func, arguments });
                    }
                    _ => {}
                }
                if let Some(new_instruction) = replacement_instruction {
                    func.dfg[instruction_id] = new_instruction;
                }
            }
        }
    }

    /// Returns the apply function for the given signature
    fn get_apply_function(&self, signature: Signature, runtime: RuntimeType) -> ApplyFunction {
        *self.apply_functions.get(&(signature, runtime)).expect("Could not find apply function")
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
    } else {
        instruction.map_values_mut(map_value);
    }

    modified
}

/// Try to map the given function literal to a field, returning Some(field) on success.
/// Returns none if the given value was not a function or doesn't need to be mapped.
fn map_function_to_field(func: &mut Function, value: ValueId) -> Option<ValueId> {
    if let Type::Function = func.dfg[value].get_type().as_ref() {
        match &func.dfg[value] {
            // If the value is a static function, transform it to the function id
            Value::Function(id) => {
                let new_value = function_id_to_field(*id);
                return Some(func.dfg.make_constant(new_value, NumericType::NativeField));
            }
            // If the value is a function used as value, just change the type of it
            Value::Instruction { .. } | Value::Param { .. } => {
                func.dfg.set_type_of_value(value, Type::field());
            }
            _ => (),
        }
    }
    None
}

/// Collects all functions used as values that can be called by their signatures
///
/// Groups all [FunctionId]s used as values by their [Signature] and [RuntimeType],
/// producing a mapping from these tuples to the list of variant functions to be dynamically dispatched.
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
    for (dispatch_signature, caller_runtime) in dynamic_dispatches {
        let target_fns =
            signature_to_functions_as_value.get(&dispatch_signature).cloned().unwrap_or_default();
        variants.insert((dispatch_signature, caller_runtime), target_fns);
    }

    // We will now have fully constructed our variants map and can return it
    variants
}

/// Finds all literal functions used as values in the given function
///
/// It is assumed that function values will only ever be used in a call instruction
/// or a store instruction.
fn find_functions_as_values(func: &Function) -> BTreeSet<FunctionId> {
    let mut functions_as_values: BTreeSet<FunctionId> = BTreeSet::new();

    let mut process_value = |value_id: ValueId| {
        if let Value::Function(id) = func.dfg[value_id] {
            functions_as_values.insert(id);
        }
    };

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];
            match instruction {
                Instruction::Call { arguments, .. } => {
                    arguments.iter().for_each(|value_id| process_value(*value_id));
                }
                Instruction::Store { value, .. } => {
                    process_value(*value);
                }
                _ => continue,
            };
        }

        block.unwrap_terminator().for_each_value(&mut process_value);
    }

    functions_as_values
}

/// Finds all dynamic dispatch signatures in the given function
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
/// # Arguments
/// - `ssa`: A mutable reference to the full [Ssa] structure containing all functions.
/// - `variants_map`:  [Variants]
///
/// # Returns
/// [ApplyFunctions]
fn create_apply_functions(ssa: &mut Ssa, variants_map: Variants) -> ApplyFunctions {
    let mut apply_functions = HashMap::default();
    for ((mut signature, runtime), variants) in variants_map.into_iter() {
        // TODO: re-enable and remove this "if variants.is_empty.."
        if variants.is_empty() {
            continue;
        }
        // assert!(
        //     !variants.is_empty(),
        //     "ICE: at least one variant should exist for a dynamic call {signature:?}"
        // );
        //
        let dispatches_to_multiple_functions = variants.len() > 1;

        // Update the shared function signature of the higher-order function variants
        // to replace any function passed as a value to a numeric field type.
        for param in &mut signature.params {
            if *param == Type::Function {
                *param = Type::field();
            }
        }

        // Update the return value types as we did for the signature parameters above.
        for ret in &mut signature.returns {
            if *ret == Type::Function {
                *ret = Type::field();
            }
        }

        let id = if dispatches_to_multiple_functions {
            // If we have multiple variants for this signature and runtime type group
            // we need to generate an apply function.
            create_apply_function(ssa, signature.clone(), runtime, variants)
        } else {
            // If there is only variant, we can use it directly rather than creating a new apply function.
            variants[0]
        };
        apply_functions
            .insert((signature, runtime), ApplyFunction { id, dispatches_to_multiple_functions });
    }
    apply_functions
}

/// Transforms a [FunctionId] into a [FieldElement]
fn function_id_to_field(function_id: FunctionId) -> FieldElement {
    (function_id.to_u32() as u128).into()
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
/// - `signature`: The shared [Signature] of all variants.
/// - `caller_runtime`: The runtime in which the apply function will be called, used to update inlining policies.
/// - `function_ids`: A non-empty list of [FunctionId]s representing concrete functions to dispatch between.
///   This method will panic if `function_ids` is empty.
///
/// # Returns
/// The [FunctionId] of the new apply function
///
/// # Panics
/// If the `function_ids` argument is empty.
fn create_apply_function(
    ssa: &mut Ssa,
    signature: Signature,
    caller_runtime: RuntimeType,
    function_ids: Vec<FunctionId>,
) -> FunctionId {
    assert!(!function_ids.is_empty());
    // Clone the user-defined globals and the function purities mapping,
    // which are shared across all functions.
    // We will be borrowing `ssa` mutably so we need to fetch this shared information
    // before attempting to add a new function to the SSA.
    let globals = ssa.main().dfg.globals.clone();
    let purities = ssa.main().dfg.function_purities.clone();
    ssa.add_fn(|id| {
        let mut function_builder = FunctionBuilder::new("apply".to_string(), id);
        function_builder.set_globals(globals);
        function_builder.set_purities(purities);

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

        let return_block = build_return_block(&mut function_builder, &signature.returns);
        // Switch back to the entry block to build the rest of the dispatch function
        function_builder.switch_to_block(entry_block);

        for (index, function_id) in function_ids.iter().enumerate() {
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

            // Jump to the return block
            function_builder.terminate_with_jmp(return_block, call_results);

            if let Some(next_block) = next_function_block {
                // Switch to the next block for the else branch
                function_builder.switch_to_block(next_block);
            }
        }
        function_builder.current_function
    })
}

/// Create the final return block for an apply function.
///
/// The return block is meant to be shared among all branches of the apply function.
/// The apply function will jump to this block after calling the appropriate
/// target function.
///
/// # Arguments
/// * `builder` - [FunctionBuilder] used to construct the function's SSA.
/// * `passed_types` - A slice of [Type]s representing the values to be returned from the function.
///
/// # Returns
/// A [BasicBlockId] representing the newly created return block.
fn build_return_block(builder: &mut FunctionBuilder, passed_types: &[Type]) -> BasicBlockId {
    let return_block = builder.insert_block();
    builder.switch_to_block(return_block);
    let params = vecmap(passed_types, |typ| builder.add_block_parameter(return_block, typ.clone()));
    builder.terminate_with_return(params);
    return_block
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn defunctionalize_missing_fn() {
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
            v4 = eq v0, Field 2
            jmpif v4 then: b3, else: b2
          b1(v2: u32):
            return v2
          b2():
            v8 = eq v0, Field 3
            jmpif v8 then: b5, else: b4
          b3():
            v6 = call f2(v1) -> u32
            jmp b1(v6)
          b4():
            constrain v0 == Field 4
            v13 = call f4(v1) -> u32
            jmp b1(v13)
          b5():
            v10 = call f3(v1) -> u32
            jmp b1(v10)
        }
        ");
    }

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
            v4 = eq v0, Field 2
            jmpif v4 then: b3, else: b2
          b1(v2: u32):
            return v2
          b2():
            v8 = eq v0, Field 3
            jmpif v8 then: b5, else: b4
          b3():
            v6 = call f2(v1) -> u32
            jmp b1(v6)
          b4():
            constrain v0 == Field 4
            v13 = call f4(v1) -> u32
            jmp b1(v13)
          b5():
            v10 = call f3(v1) -> u32
            jmp b1(v10)
        }
        ");
    }

    #[test]
    fn apply_created_per_caller_runtime() {
        let src = "
          acir(inline) fn main f0 {
            b0(v0: u32):
              v3 = call f1(f2, v0) -> u32
              v5 = add v0, u32 1
              v6 = eq v3, v5
              constrain v3 == v5
              v9 = call f4(f3, v0) -> u32
              v10 = add v0, u32 1
              v11 = eq v9, v10
              constrain v9 == v10
              return
          }
          brillig(inline) fn wrapper f1 {
            b0(v0: function, v1: u32):
              v2 = call v0(v1) -> u32
              return v2
          }
          acir(inline) fn wrapper_acir f4 {
            b0(v0: function, v1: u32):
              v2 = call v0(v1) -> u32
              return v2
          }
          brillig(inline) fn increment f2 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
          acir(inline) fn increment_acir f3 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        let applies = ssa.functions.values().filter(|f| f.name() == "apply").collect::<Vec<_>>();
        assert_eq!(applies.len(), 2);
        assert!(applies.iter().any(|f| f.runtime().is_acir()));
        assert!(applies.iter().any(|f| f.runtime().is_brillig()));
    }
}
