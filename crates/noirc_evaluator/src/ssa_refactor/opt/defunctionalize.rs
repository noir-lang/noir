use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType},
        instruction::{BinaryOp, Instruction, TerminatorInstruction},
        types::{NumericType, Type},
        value::{Value, ValueId},
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

/// Performs defunctionalization on all functions
/// This is done by changing all functions as value to be a number (FieldElement)
/// And creating apply functions that dispatch to the correct target by runtime comparisons with constants
#[derive(Debug, Clone)]
struct DefunctionalizationContext {
    fn_to_runtime: HashMap<FunctionId, RuntimeType>,
    variants: HashMap<FunctionSignature, Vec<FunctionId>>,
    apply_functions: HashMap<FunctionSignature, FunctionId>,
}

impl DefunctionalizationContext {
    /// Returns the new ssa with the function defunctionalized
    pub(crate) fn defunctionalize_ssa(mut ssa: Ssa) -> Ssa {
        // Find all functions that share the signature
        let variants = find_variants(&ssa);
        // Create apply functions
        let apply_functions = create_apply_functions(&mut ssa, &variants);
        let fn_to_runtime = ssa
            .functions
            .iter()
            .map(|(func_id, func)| (*func_id, func.runtime()))
            .collect::<HashMap<_, _>>();

        let context = DefunctionalizationContext { fn_to_runtime, variants, apply_functions };

        context.defunctionalize_all(ssa)
    }

    /// Defunctionalize all functions in the Ssa
    fn defunctionalize_all(mut self, mut ssa: Ssa) -> Ssa {
        let func_ids = ssa.functions.keys().copied().collect::<Vec<_>>();
        for func_id in func_ids {
            ssa = self.defunctionalize(func_id, ssa);
        }
        ssa
    }

    /// Defunctionalize a single function
    fn defunctionalize(&mut self, func_id: FunctionId, mut ssa: Ssa) -> Ssa {
        let func = ssa.get_fn_mut(func_id);
        let mut target_function_ids = HashSet::new();

        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let instructions = block.instructions().to_vec();

            for instruction_id in instructions {
                let instruction = func.dfg[instruction_id].clone();
                let mut new_instruction = None;
                // Operate on call instructions
                if let Instruction::Call { func: target_func_id, arguments } = instruction {
                    match func.dfg[target_func_id] {
                        // If the target is a function used as value
                        Value::Param { .. } | Value::Instruction { .. } => {
                            // Collect the argument types
                            let argument_types =
                                vecmap(arguments.to_owned(), |arg| func.dfg.type_of_value(arg));

                            // Collect the result types
                            let result_types = vecmap(
                                func.dfg.instruction_results(instruction_id).to_owned(),
                                |result| func.dfg.type_of_value(result),
                            );
                            // Find the correct apply function
                            let apply_function = self.get_apply_function(&FunctionSignature {
                                parameters: argument_types,
                                returns: result_types,
                                runtime: func.runtime(),
                            });
                            target_function_ids.insert(apply_function);

                            // Replace the instruction with a call to apply
                            let apply_function = func.dfg.import_function(apply_function);
                            let mut new_arguments = vec![target_func_id];
                            new_arguments.extend(arguments);
                            new_instruction = Some(Instruction::Call {
                                func: apply_function,
                                arguments: new_arguments,
                            });
                        }
                        Value::Function(id) => {
                            target_function_ids.insert(id);
                        }
                        _ => {}
                    }
                }
                if let Some(new_instruction) = new_instruction {
                    func.dfg[instruction_id] = new_instruction;
                }
            }
        }

        // Change the type of all the values that are not call targets to NativeField
        for value_id in func.dfg.value_ids() {
            let value = &mut func.dfg[value_id];
            if let Type::Function = value.get_type() {
                // If the value is a static function, transform it to the function id
                if let Value::Function(id) = value {
                    let id = *id;
                    if !target_function_ids.contains(&id) {
                        *value = Value::NumericConstant {
                            constant: (id.to_usize() as u128).into(),
                            typ: Type::Numeric(NumericType::NativeField),
                        }
                    }
                }
                // If it is a dynamic function, just change the type
                if let Value::Instruction { typ, .. } | Value::Param { typ, .. } = value {
                    *typ = Type::Numeric(NumericType::NativeField);
                }
            }
        }

        ssa
    }

    /// Returns the apply function for the given signature
    fn get_apply_function(&self, signature: &FunctionSignature) -> FunctionId {
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
        let function = ssa.get_fn(function_id);
        let signature = FunctionSignature::from(function);
        variants.entry(signature).or_default().push(function_id);
    }

    variants
}

/// Finds all literal functions used as values in the given function
fn functions_as_values(func: &Function) -> HashSet<FunctionId> {
    let mut functions = HashSet::new();

    let mut append_functions = |values: &[ValueId]| {
        for value in values {
            if let Value::Function(id) = func.dfg[*value] {
                functions.insert(id);
            }
        }
    };

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];
            match instruction {
                Instruction::Call { arguments, .. } => {
                    append_functions(arguments);
                }
                Instruction::Store { value, .. } => {
                    append_functions(&[*value]);
                }
                _ => {}
            }
        }
        match block.terminator() {
            Some(TerminatorInstruction::Jmp { arguments, .. }) => {
                append_functions(arguments);
            }
            Some(TerminatorInstruction::Return { return_values }) => {
                append_functions(return_values);
            }
            _ => {}
        }
    }
    functions
}

fn create_apply_functions(
    ssa: &mut Ssa,
    variants_map: &HashMap<FunctionSignature, Vec<FunctionId>>,
) -> HashMap<FunctionSignature, FunctionId> {
    let mut apply_functions = HashMap::new();
    for (signature, variants) in variants_map.iter() {
        let apply_function = create_apply_function(ssa, signature, variants);
        apply_functions.insert(signature.clone(), apply_function);
    }
    apply_functions
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
        let target_id = function_builder.add_parameter(Type::Numeric(NumericType::NativeField));
        let params_ids =
            vecmap(signature.parameters.clone(), |typ| function_builder.add_parameter(typ));

        let mut previous_target_block = None;
        for (index, function_id) in function_ids.iter().enumerate() {
            let is_last = index == function_ids.len() - 1;
            let mut next_function_block = None;

            let function_id_constant = function_builder.numeric_constant(
                function_id.to_usize() as u128,
                Type::Numeric(NumericType::NativeField),
            );
            let condition =
                function_builder.insert_binary(target_id, BinaryOp::Eq, function_id_constant);

            // If it's not the last function to dispatch, crate an if statement
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
            let target_block = match previous_target_block {
                Some(block) => {
                    let current_block = function_builder.current_block();
                    build_return_block(
                        &mut function_builder,
                        current_block,
                        signature.returns.clone(),
                        Some(block),
                    )
                }
                None => {
                    let current_block = function_builder.current_block();
                    build_return_block(
                        &mut function_builder,
                        current_block,
                        signature.returns.clone(),
                        None,
                    )
                }
            };
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

impl Ssa {
    pub(crate) fn defunctionalize(self) -> Ssa {
        DefunctionalizationContext::defunctionalize_ssa(self)
    }
}
