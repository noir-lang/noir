//! This module defines the defunctionalization pass for the SSA IR.
//! The purpose of this pass is to transforms all functions used as values into
//! constant numbers (fields) that represent the function id. That way all calls
//! with a non-literal target can be replaced with a call to an apply function.
//! The apply function is a dispatch function that takes the function id as a parameter
//! and dispatches to the correct target.
use std::collections::{HashMap, HashSet};

use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType},
        instruction::{BinaryOp, Instruction},
        types::{NumericType, Type},
        value::Value,
    },
    ssa_builder::FunctionBuilder,
    ssa_gen::Ssa,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FunctionSignature {
    parameters: Vec<Type>,
    returns: Vec<Type>,
    runtime: RuntimeType,
}

impl FunctionSignature {
    fn from(function: &Function) -> Self {
        let parameters = vecmap(function.parameters(), |param| function.dfg.type_of_value(*param));
        let returns = vecmap(function.returns(), |ret| function.dfg.type_of_value(*ret));
        let runtime = function.runtime();
        Self { parameters, returns, runtime }
    }
}

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

/// Performs defunctionalization on all functions
/// This is done by changing all functions as value to be a number (FieldElement)
/// And creating apply functions that dispatch to the correct target by runtime comparisons with constants
#[derive(Debug, Clone)]
struct DefunctionalizationContext {
    fn_to_runtime: HashMap<FunctionId, RuntimeType>,
    variants: HashMap<FunctionSignature, Vec<FunctionId>>,
    apply_functions: HashMap<FunctionSignature, ApplyFunction>,
}

impl Ssa {
    pub(crate) fn defunctionalize(mut self) -> Ssa {
        // Find all functions used as value that share the same signature
        let variants = find_variants(&self);

        let apply_functions = create_apply_functions(&mut self, &variants);
        let fn_to_runtime =
            self.functions.iter().map(|(func_id, func)| (*func_id, func.runtime())).collect();

        let context = DefunctionalizationContext { fn_to_runtime, variants, apply_functions };

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
        let mut target_function_ids = HashSet::new();

        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let instructions = block.instructions().to_vec();

            for instruction_id in instructions {
                let instruction = func.dfg[instruction_id].clone();
                let mut replacement_instruction = None;
                // Operate on call instructions
                let (target_func_id, mut arguments) = match instruction {
                    Instruction::Call { func: target_func_id, arguments } => {
                        (target_func_id, arguments)
                    }
                    _ => continue,
                };

                match func.dfg[target_func_id] {
                    // If the target is a function used as value
                    Value::Param { .. } | Value::Instruction { .. } => {
                        // Collect the argument types
                        let argument_types = vecmap(&arguments, |arg| func.dfg.type_of_value(*arg));

                        // Collect the result types
                        let result_types =
                            vecmap(func.dfg.instruction_results(instruction_id), |result| {
                                func.dfg.type_of_value(*result)
                            });
                        // Find the correct apply function
                        let apply_function = self.get_apply_function(&FunctionSignature {
                            parameters: argument_types,
                            returns: result_types,
                            runtime: func.runtime(),
                        });
                        target_function_ids.insert(apply_function.id);

                        // Replace the instruction with a call to apply
                        let apply_function_value_id = func.dfg.import_function(apply_function.id);
                        if apply_function.dispatches_to_multiple_functions {
                            arguments.insert(0, target_func_id);
                        }
                        let func = apply_function_value_id;
                        replacement_instruction = Some(Instruction::Call { func, arguments });
                    }
                    Value::Function(id) => {
                        target_function_ids.insert(id);
                    }
                    _ => {}
                }
                if let Some(new_instruction) = replacement_instruction {
                    func.dfg[instruction_id] = new_instruction;
                }
            }
        }

        // Change the type of all the values that are not call targets to NativeField
        let value_ids = vecmap(func.dfg.values_iter(), |(id, _)| id);
        for value_id in value_ids {
            if let Type::Function = &func.dfg[value_id].get_type() {
                match &func.dfg[value_id] {
                    // If the value is a static function, transform it to the function id
                    Value::Function(id) => {
                        if !target_function_ids.contains(id) {
                            let new_value =
                                func.dfg.make_constant(function_id_to_field(*id), Type::field());
                            func.dfg.set_value_from_id(value_id, new_value);
                        }
                    }
                    // If the value is a function used as value, just change the type of it
                    Value::Instruction { .. } | Value::Param { .. } => {
                        func.dfg.set_type_of_value(value_id, Type::field());
                    }
                    _ => {}
                }
            }
        }
    }

    /// Returns the apply function for the given signature
    fn get_apply_function(&self, signature: &FunctionSignature) -> ApplyFunction {
        *self.apply_functions.get(signature).expect("Could not find apply function")
    }
}

/// Collects all functions used as a value by their signatures
fn find_variants(ssa: &Ssa) -> HashMap<FunctionSignature, Vec<FunctionId>> {
    let mut variants: HashMap<FunctionSignature, Vec<FunctionId>> = HashMap::new();
    let mut functions_used_as_values = HashSet::new();

    for function in ssa.functions.values() {
        functions_used_as_values.extend(functions_as_values(function));
    }

    for function_id in functions_used_as_values {
        let function = &ssa.functions[&function_id];
        let signature = FunctionSignature::from(function);
        variants.entry(signature).or_default().push(function_id);
    }

    variants
}

/// Finds all literal functions used as values in the given function
fn functions_as_values(func: &Function) -> HashSet<FunctionId> {
    let mut literal_functions: HashSet<_> = func
        .dfg
        .values_iter()
        .filter_map(|(id, _)| match func.dfg[id] {
            Value::Function(id) => Some(id),
            _ => None,
        })
        .collect();

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];
            let target_value = match instruction {
                Instruction::Call { func, .. } => func,
                _ => continue,
            };
            let target_id = match func.dfg[*target_value] {
                Value::Function(id) => id,
                _ => continue,
            };
            literal_functions.remove(&target_id);
        }
    }
    literal_functions
}

fn create_apply_functions(
    ssa: &mut Ssa,
    variants_map: &HashMap<FunctionSignature, Vec<FunctionId>>,
) -> HashMap<FunctionSignature, ApplyFunction> {
    let mut apply_functions = HashMap::new();
    for (signature, variants) in variants_map.iter() {
        let dispatches_to_multiple_functions = variants.len() > 1;
        let id = if dispatches_to_multiple_functions {
            create_apply_function(ssa, signature, variants)
        } else {
            variants[0]
        };
        apply_functions
            .insert(signature.clone(), ApplyFunction { id, dispatches_to_multiple_functions });
    }
    apply_functions
}

fn function_id_to_field(function_id: FunctionId) -> FieldElement {
    (function_id.to_usize() as u128).into()
}

/// Creates an apply function for the given signature and variants
fn create_apply_function(
    ssa: &mut Ssa,
    signature: &FunctionSignature,
    function_ids: &[FunctionId],
) -> FunctionId {
    assert!(!function_ids.is_empty());
    ssa.add_fn(|id| {
        let mut function_builder = FunctionBuilder::new("apply".to_string(), id, signature.runtime);
        let target_id = function_builder.add_parameter(Type::field());
        let params_ids =
            vecmap(signature.parameters.clone(), |typ| function_builder.add_parameter(typ));

        let mut previous_target_block = None;
        for (index, function_id) in function_ids.iter().enumerate() {
            let is_last = index == function_ids.len() - 1;
            let mut next_function_block = None;

            let function_id_constant = function_builder.numeric_constant(
                function_id_to_field(*function_id),
                Type::Numeric(NumericType::NativeField),
            );
            let condition =
                function_builder.insert_binary(target_id, BinaryOp::Eq, function_id_constant);

            // If it's not the last function to dispatch, create an if statement
            if !is_last {
                next_function_block = Some(function_builder.insert_block());
                let executor_block = function_builder.insert_block();

                function_builder.terminate_with_jmpif(
                    condition,
                    executor_block,
                    next_function_block.unwrap(),
                );
                function_builder.switch_to_block(executor_block);
            } else {
                // Else just constrain the condition
                function_builder.insert_constrain(condition);
            }
            // Find the target block or build it if necessary
            let current_block = function_builder.current_block();

            let target_block = build_return_block(
                &mut function_builder,
                current_block,
                signature.returns.clone(),
                previous_target_block,
            );
            previous_target_block = Some(target_block);

            // Call the function
            let target_function_value = function_builder.import_function(*function_id);
            let call_results = function_builder
                .insert_call(target_function_value, params_ids.clone(), signature.returns.clone())
                .to_vec();

            // Jump to the target block for returning
            function_builder.terminate_with_jmp(target_block, call_results);

            if let Some(next_block) = next_function_block {
                // Switch to the next block for the else branch
                function_builder.switch_to_block(next_block);
            }
        }
        function_builder.current_function
    })
}

/// Crates a return block, if no previous return exists, it will create a final return
/// Else, it will create a bypass return block that points to the previous return block
fn build_return_block(
    builder: &mut FunctionBuilder,
    previous_block: BasicBlockId,
    passed_types: Vec<Type>,
    target: Option<BasicBlockId>,
) -> BasicBlockId {
    let return_block = builder.insert_block();
    builder.switch_to_block(return_block);

    let params = vecmap(passed_types, |typ| builder.add_block_parameter(return_block, typ));
    match target {
        None => builder.terminate_with_return(params),
        Some(target) => builder.terminate_with_jmp(target, params),
    }
    builder.switch_to_block(previous_block);
    return_block
}
