//! This module defines the defunctionalization pass for the SSA IR.
//! The purpose of this pass is to transforms all functions used as values into
//! constant numbers (fields) that represent the function id. That way all calls
//! with a non-literal target can be replaced with a call to an apply function.
//! The apply function is a dispatch function that takes the function id as a parameter
//! and dispatches to the correct target.
use std::collections::{BTreeMap, BTreeSet, HashSet};

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

type Variants = BTreeMap<(Signature, RuntimeType), Vec<FunctionId>>;
type ApplyFunctions = HashMap<(Signature, RuntimeType), ApplyFunction>;

/// Performs defunctionalization on all functions
/// This is done by changing all functions as value to be a number (FieldElement)
/// And creating apply functions that dispatch to the correct target by runtime comparisons with constants
#[derive(Debug, Clone)]
struct DefunctionalizationContext {
    apply_functions: ApplyFunctions,
}

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn defunctionalize(mut self) -> Ssa {
        // Find all functions used as value that share the same signature
        let variants = find_variants(&self);

        let apply_functions = create_apply_functions(&mut self, variants);

        let context = DefunctionalizationContext { apply_functions };

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
        let mut call_target_values = HashSet::new();

        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let instructions = block.instructions().to_vec();

            for instruction_id in instructions {
                let instruction = func.dfg[instruction_id].clone();
                let mut replacement_instruction = None;
                // Operate on call instructions
                let (target_func_id, arguments) = match &instruction {
                    Instruction::Call { func: target_func_id, arguments } => {
                        (*target_func_id, arguments)
                    }
                    _ => continue,
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
                        call_target_values.insert(func);

                        replacement_instruction = Some(Instruction::Call { func, arguments });
                    }
                    Value::Function(..) => {
                        call_target_values.insert(target_func_id);
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
            if let Type::Function = func.dfg[value_id].get_type().as_ref() {
                match &func.dfg[value_id] {
                    // If the value is a static function, transform it to the function id
                    Value::Function(id) => {
                        if !call_target_values.contains(&value_id) {
                            let field = NumericType::NativeField;
                            let new_value =
                                func.dfg.make_constant(function_id_to_field(*id), field);
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
    fn get_apply_function(&self, signature: Signature, runtime: RuntimeType) -> ApplyFunction {
        *self.apply_functions.get(&(signature, runtime)).expect("Could not find apply function")
    }
}

/// Collects all functions used as values that can be called by their signatures
fn find_variants(ssa: &Ssa) -> Variants {
    let mut dynamic_dispatches: BTreeSet<(Signature, RuntimeType)> = BTreeSet::new();
    let mut functions_as_values: BTreeSet<FunctionId> = BTreeSet::new();

    for function in ssa.functions.values() {
        functions_as_values.extend(find_functions_as_values(function));
        dynamic_dispatches.extend(
            find_dynamic_dispatches(function).into_iter().map(|sig| (sig, function.runtime())),
        );
    }

    let mut signature_to_functions_as_value: BTreeMap<Signature, Vec<FunctionId>> = BTreeMap::new();

    for function_id in functions_as_values {
        let signature = ssa.functions[&function_id].signature();
        signature_to_functions_as_value.entry(signature).or_default().push(function_id);
    }

    let mut variants: Variants = BTreeMap::new();

    for (dispatch_signature, caller_runtime) in dynamic_dispatches {
        let target_fns =
            signature_to_functions_as_value.get(&dispatch_signature).cloned().unwrap_or_default();
        variants.insert((dispatch_signature, caller_runtime), target_fns);
    }

    variants
}

/// Finds all literal functions used as values in the given function
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

fn create_apply_functions(
    ssa: &mut Ssa,
    variants_map: BTreeMap<(Signature, RuntimeType), Vec<FunctionId>>,
) -> ApplyFunctions {
    let mut apply_functions = HashMap::default();
    for ((signature, runtime), variants) in variants_map.into_iter() {
        assert!(
            !variants.is_empty(),
            "ICE: at least one variant should exist for a dynamic call {signature:?}"
        );
        let dispatches_to_multiple_functions = variants.len() > 1;

        let id = if dispatches_to_multiple_functions {
            create_apply_function(ssa, signature.clone(), runtime, variants)
        } else {
            variants[0]
        };
        apply_functions
            .insert((signature, runtime), ApplyFunction { id, dispatches_to_multiple_functions });
    }
    apply_functions
}

fn function_id_to_field(function_id: FunctionId) -> FieldElement {
    (function_id.to_u32() as u128).into()
}

/// Creates an apply function for the given signature and variants
fn create_apply_function(
    ssa: &mut Ssa,
    signature: Signature,
    caller_runtime: RuntimeType,
    function_ids: Vec<FunctionId>,
) -> FunctionId {
    assert!(!function_ids.is_empty());
    let globals = ssa.functions[&function_ids[0]].dfg.globals.clone();
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
        let target_id = function_builder.add_parameter(Type::field());
        let params_ids = vecmap(signature.params, |typ| function_builder.add_parameter(typ));

        let mut previous_target_block = None;
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
            // Find the target block or build it if necessary
            let current_block = function_builder.current_block();

            let target_block = build_return_block(
                &mut function_builder,
                current_block,
                &signature.returns,
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

/// If no previous return target exists, it will create a final return,
/// otherwise returns the existing return block to jump to.
fn build_return_block(
    builder: &mut FunctionBuilder,
    previous_block: BasicBlockId,
    passed_types: &[Type],
    target: Option<BasicBlockId>,
) -> BasicBlockId {
    if let Some(return_block) = target {
        return return_block;
    }
    let return_block = builder.insert_block();
    builder.switch_to_block(return_block);
    let params = vecmap(passed_types, |typ| builder.add_block_parameter(return_block, typ.clone()));
    builder.terminate_with_return(params);
    builder.switch_to_block(previous_block);
    return_block
}

#[cfg(test)]
mod tests {
    use crate::ssa::opt::assert_normalized_ssa_equals;

    use super::Ssa;

    #[test]
    fn apply_inherits_caller_runtime() {
        // Extracted from `execution_success/brillig_fns_as_values` with `--force-brillig`
        let src = "
          brillig(inline) fn main f0 {
            b0(v0: u32):
              v3 = call f1(f2, v0) -> u32
              v5 = add v0, u32 1
              v6 = eq v3, v5
              constrain v3 == v5
              v9 = call f1(f3, v0) -> u32
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
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();

        let expected = "
          brillig(inline) fn main f0 {
            b0(v0: u32):
              v3 = call f1(Field 2, v0) -> u32
              v5 = add v0, u32 1
              v6 = eq v3, v5
              constrain v3 == v5
              v9 = call f1(Field 3, v0) -> u32
              v10 = add v0, u32 1
              v11 = eq v9, v10
              constrain v9 == v10
              return
          }
          brillig(inline) fn wrapper f1 {
            b0(v0: Field, v1: u32):
              v3 = call f4(v0, v1) -> u32
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
          brillig(inline_always) fn apply f4 {
            b0(v0: Field, v1: u32):
              v4 = eq v0, Field 2
              jmpif v4 then: b2, else: b1
            b1():
              constrain v0 == Field 3
              v7 = call f3(v1) -> u32
              jmp b3(v7)
            b2():
              v9 = call f2(v1) -> u32
              jmp b3(v9)
            b3(v2: u32):
              return v2
          }
        ";
        assert_normalized_ssa_equals(ssa, expected);
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
