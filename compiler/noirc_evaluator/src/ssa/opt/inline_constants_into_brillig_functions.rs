use std::collections::HashMap;

use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::Instruction,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn inline_constants_into_brillig_functions(mut self) -> Ssa {
        // We first gather all calls to brillig functions that have some constants in them,
        // together with how many calls were done to it (in total, and with a given set of constants)

        // Calls to a given function with arguments where some might be constants
        // function_id -> (constants -> count)
        let mut calls = HashMap::new();

        // Count of all calls to a given function
        // function_id -> count
        let mut total_calls = HashMap::new();

        for function in self.functions.values() {
            function.gather_calls_to_brillig_functions_with_constants(
                &self,
                &mut calls,
                &mut total_calls,
            );
        }

        // Now we determine which constants we are going to inline.
        // The rule we'll use is: if a given set of constants was used more than 30%
        // of the time across all calls to a given function, we'll create a specific
        // function with those constants inlined.
        calls.retain(|func_id, entries| {
            let total_count = total_calls[func_id] as f64;
            entries.retain(|_, count| (*count as f64 / total_count) >= 0.3);
            !entries.is_empty()
        });

        // Next, create specialized functions where those constants are inlined
        // function_id -> (constants -> new_function_id)
        let mut new_functions: HashMap<FunctionId, HashMap<Vec<Option<Constant>>, FunctionId>> =
            HashMap::new();

        for (func_id, entries) in calls {
            let function = self.functions[&func_id].clone();
            for (constants, _) in entries {
                let new_function_id = self.add_fn(|func_id| {
                    inline_constants_into_function(&function, &constants, func_id)
                });
                let entry = new_functions.entry(func_id).or_default();
                entry.entry(constants).insert_entry(new_function_id);
            }
        }

        // Finally, redirect calls to use the new functions
        for function in self.functions.values_mut() {
            function.replace_brillig_calls_with_constants(&new_functions);
        }

        self
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Constant {
    Number(FieldElement, NumericType),
    Array(Vec<Constant>, Type),
}

impl Function {
    fn gather_calls_to_brillig_functions_with_constants(
        &self,
        ssa: &Ssa,
        calls: &mut HashMap<FunctionId, HashMap<Vec<Option<Constant>>, usize>>,
        total_calls: &mut HashMap<FunctionId, usize>,
    ) {
        for block in self.reachable_blocks() {
            for instruction_id in self.dfg[block].instructions() {
                let instruction = &self.dfg[*instruction_id];
                let Instruction::Call { func, arguments } = instruction else {
                    continue;
                };

                let Value::Function(func_id) = self.dfg[*func] else {
                    continue;
                };

                let func = &ssa.functions[&func_id];
                if !func.runtime().is_brillig() {
                    continue;
                }

                *total_calls.entry(func_id).or_default() += 1;

                if !arguments.iter().any(|argument| self.dfg.is_constant(*argument)) {
                    continue;
                }

                let constants = vecmap(arguments, |argument| get_constant(*argument, &self.dfg));
                *calls.entry(func_id).or_default().entry(constants).or_default() += 1;
            }
        }
    }

    fn replace_brillig_calls_with_constants(
        &mut self,
        functions: &HashMap<FunctionId, HashMap<Vec<Option<Constant>>, FunctionId>>,
    ) {
        for block in self.reachable_blocks() {
            let instruction_ids = self.dfg[block].take_instructions();
            for instruction_id in instruction_ids {
                let instruction = &self.dfg[instruction_id];
                let Instruction::Call { func, arguments } = instruction else {
                    self.dfg[block].insert_instruction(instruction_id);
                    continue;
                };

                let Value::Function(func_id) = self.dfg[*func] else {
                    self.dfg[block].insert_instruction(instruction_id);
                    continue;
                };

                let Some(entries) = functions.get(&func_id) else {
                    self.dfg[block].insert_instruction(instruction_id);
                    continue;
                };

                if !arguments.iter().any(|argument| self.dfg.is_constant(*argument)) {
                    self.dfg[block].insert_instruction(instruction_id);
                    continue;
                }

                let constants = vecmap(arguments, |argument| get_constant(*argument, &self.dfg));
                let Some(new_function_id) = entries.get(&constants) else {
                    self.dfg[block].insert_instruction(instruction_id);
                    continue;
                };

                let mut new_arguments = Vec::new();
                for (index, constant) in constants.iter().enumerate() {
                    if constant.is_none() {
                        new_arguments.push(arguments[index]);
                    }
                }

                let new_function_id = self.dfg.import_function(*new_function_id);
                let new_instruction =
                    Instruction::Call { func: new_function_id, arguments: new_arguments };
                let call_stack = self.dfg.get_instruction_call_stack_id(instruction_id);
                let old_results = self.dfg.instruction_results(instruction_id);
                let old_results = old_results.to_vec();
                let typevars = old_results
                    .iter()
                    .map(|value| self.dfg.type_of_value(*value))
                    .collect::<Vec<_>>();

                let new_results = self.dfg.insert_instruction_and_results(
                    new_instruction,
                    block,
                    Some(typevars),
                    call_stack,
                );
                let new_results = new_results.results().iter().copied().collect::<Vec<_>>();
                for (old_result, new_result) in old_results.into_iter().zip(new_results) {
                    self.dfg.set_value_from_id(old_result, new_result);
                }
            }
        }
    }
}

fn get_constant(value: ValueId, dfg: &DataFlowGraph) -> Option<Constant> {
    if let Some((value, typ)) = dfg.get_numeric_constant_with_type(value) {
        return Some(Constant::Number(value, typ));
    }

    if let Some((values, typ)) = dfg.get_array_constant(value) {
        let mut constants = Vec::with_capacity(values.len());
        for value in values {
            constants.push(get_constant(value, dfg)?);
        }
        return Some(Constant::Array(constants, typ));
    }

    None
}

fn inline_constants_into_function(
    function: &Function,
    constants: &[Option<Constant>],
    id: FunctionId,
) -> Function {
    let mut function = Function::clone_with_id(id, function);
    let parameters = function.parameters().to_vec();

    // First replace all constant parameters
    for (parameter, constant) in parameters.iter().zip(constants) {
        if let Some(constant) = constant {
            let constant = make_constant(&mut function.dfg, constant);
            function.dfg.set_value_from_id(*parameter, constant);
        }
    }

    // Then keep only those parameters for which the argument is not a constant
    let mut new_parameters = Vec::new();
    for (index, constant) in constants.iter().enumerate() {
        if constant.is_none() {
            new_parameters.push(parameters[index]);
        }
    }
    let entry_block_id = function.entry_block();
    let entry_block = &mut function.dfg[entry_block_id];
    entry_block.set_parameters(new_parameters);

    function
}

fn make_constant(dfg: &mut DataFlowGraph, constant: &Constant) -> ValueId {
    match constant {
        Constant::Number(value, typ) => dfg.make_constant(*value, *typ),
        Constant::Array(_, _) => todo!("Handle arrays"),
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa};

    #[test]
    fn inlines_if_same_constant_is_always_used() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v3 = call f1(Field 1, v0) -> Field
            v4 = call f1(Field 1, v0) -> Field
            v5 = add v3, v4
            return v5
        }
        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f2(v0) -> Field
            v3 = call f2(v0) -> Field
            v4 = add v2, v3
            return v4
        }
        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        brillig(inline) fn foo f2 {
          b0(v0: Field):
            v2 = add Field 1, v0
            return v2
        }
        ";
        let ssa = ssa.inline_constants_into_brillig_functions();
        assert_normalized_ssa_equals(ssa, expected);
    }
}
